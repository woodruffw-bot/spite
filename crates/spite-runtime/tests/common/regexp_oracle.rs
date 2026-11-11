//! Reviewed native-result comparisons for RegExp behavior.

use spite_core::JsString;
use spite_runtime::{Realm, Value};

include!("regexp_oracle_data.rs");

const DESCRIBE: &str = r#"
function describe(v) {
    if (v === undefined) return ['undefined'];
    if (v === null) return ['null'];
    if (typeof v === 'number') {
        let value = v;
        if (Number.isNaN(v)) value = 'NaN';
        else if (Object.is(v, -0)) value = '-0';
        else if (v === Infinity) value = 'Infinity';
        else if (v === -Infinity) value = '-Infinity';
        return ['number', value];
    }
    if (typeof v === 'bigint') return ['bigint', String(v)];
    if (typeof v === 'function') return ['function', v.name, v.length];
    if (typeof v !== 'object') return [typeof v, v];
    if (v instanceof RegExp) return ['regexp', v.source, v.flags, v.lastIndex];
    if (Array.isArray(v)) {
        return [
            'array', Array.from(v, describe), describe(v.index),
            describe(v.input), describe(v.groups), describe(v.indices)
        ];
    }
    return ['object', Object.keys(v).map(function(key) {
        return [key, describe(v[key])];
    })];
}
"#;

#[track_caller]
pub fn assert_matches_reference(realm: &mut Realm, source: &str) {
    let mut quoted = String::from("\"");
    for unit in source.encode_utf16() {
        use std::fmt::Write;
        write!(quoted, "\\u{unit:04x}").unwrap();
    }
    quoted.push('"');
    let program = format!(
        r#"
        JSON.stringify((function() {{
            {DESCRIBE}
            try {{
                return describe(eval({quoted}));
            }} catch (error) {{
                return ['throw', error.name];
            }}
        }})())
        "#
    );
    let expected = if let Ok(path) = std::env::var("SPITE_REGEXP_ORACLE_CAPTURE") {
        // Developer fixture generation still compares the actual engine result;
        // this mode never substitutes success or changes product limits.
        let output = std::process::Command::new("node")
            .args([
                "--eval",
                "process.stdout.write(String(eval(process.argv[1])))",
                &program,
            ])
            .output()
            .expect("native oracle generation requires Node");
        assert!(
            output.status.success(),
            "native oracle failed: {source}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let expected = String::from_utf8(output.stdout).unwrap();
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let hex = |text: &str| {
            text.as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        writeln!(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap(),
            "{}\t{}",
            hex(source),
            hex(&expected)
        )
        .unwrap();
        expected
    } else {
        let index = EXPECTATIONS
            .binary_search_by(|(candidate, _)| candidate.cmp(&source))
            .unwrap_or_else(|_| panic!("missing reviewed native expectation for: {source}"));
        EXPECTATIONS[index].1.to_owned()
    };
    assert_eq!(
        realm.eval(&program),
        Ok(Value::String(JsString::from(expected.as_str()))),
        "{source}"
    );
}
