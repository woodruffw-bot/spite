//! Static native-result checks replacing obsolete Unsupported expectations.

use spite_core::JsString;
use spite_runtime::{Realm, Value};

include!("regexp_oracle_data.rs");

const DESCRIBE: &str = r#"
function describe(v) {
    if (v === undefined) return ['undefined'];
    if (v === null) return ['null'];
    if (typeof v === 'number') return ['number',Number.isNaN(v)?'NaN':Object.is(v,-0)?'-0':v===Infinity?'Infinity':v===-Infinity?'-Infinity':v];
    if (typeof v === 'bigint') return ['bigint',String(v)];
    if (typeof v === 'function') return ['function',v.name,v.length];
    if (typeof v !== 'object') return [typeof v,v];
    if (v instanceof RegExp) return ['regexp',v.source,v.flags,v.lastIndex];
    if (Array.isArray(v)) return ['array',Array.from(v,describe),describe(v.index),describe(v.input),describe(v.groups),describe(v.indices)];
    return ['object',Object.keys(v).map(function(key){return [key,describe(v[key])]})];
}
"#;

pub fn check(realm: &mut Realm, source: &str) -> bool {
    let mut quoted = String::from("\"");
    for unit in source.encode_utf16() {
        use std::fmt::Write;
        write!(quoted, "\\u{unit:04x}").unwrap();
    }
    quoted.push('"');
    let program = format!(
        "JSON.stringify((function(){{{DESCRIBE}try{{return describe(eval({quoted}));}}catch(error){{return ['throw',error.name];}}}})())"
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
        use std::{
            fs::OpenOptions,
            io::Write,
            sync::{Mutex, OnceLock},
        };
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
    true
}
