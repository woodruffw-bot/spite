//! Reviewed upstream parser regressions, not full Test262 harness executions.

use spite_parser::{ast::*, parse_script};
use std::{collections::BTreeSet, fs, path::Path};

#[test]
fn pinned_let_statement_lookahead() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test262");
    let manifest = fs::read_to_string(root.join("manifest.tsv")).unwrap();
    let expected = BTreeSet::from([
        "test/language/statements/while/let-block-with-newline.js",
        "test/language/statements/while/let-identifier-with-newline.js",
        "test/language/statements/if/let-block-with-newline.js",
        "test/language/statements/if/let-identifier-with-newline.js",
    ]);
    let mut checked = BTreeSet::new();
    for line in manifest
        .lines()
        .filter(|line| line.starts_with("parser-pass\t"))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let path = fields[2];
        assert!(expected.contains(path), "unreviewed parser fixture: {path}");
        assert!(checked.insert(path), "duplicate parser fixture: {path}");
        let source = fs::read_to_string(root.join("upstream").join(path)).unwrap();
        assert!(source.contains("\nflags: [noStrict]\n"));
        assert!(!source.contains("\nnegative:\n"));
        // Parse the complete original source, without stripping or rewriting it.
        let script = parse_script(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(!script.is_strict());
        assert_eq!(
            script.statements().len(),
            2,
            "{path}: ASI must split the statements"
        );
        let body = match &script.statements()[0].kind {
            StatementKind::While { body, .. } => body,
            StatementKind::If { consequent, .. } => consequent,
            _ => panic!("{path}: expected while or if"),
        };
        assert!(
            matches!(&body.kind, StatementKind::Expression(Expr { kind: ExprKind::Identifier(name), .. }) if name == "let")
        );
        if path.ends_with("let-block-with-newline.js") {
            assert!(
                matches!(&script.statements()[1].kind, StatementKind::Block(body) if body.is_empty())
            );
        } else {
            assert!(
                matches!(&script.statements()[1].kind, StatementKind::Expression(Expr { kind: ExprKind::Assign(name, _), .. }) if name == "x")
            );
        }
    }
    assert_eq!(checked, expected, "reviewed parser inventory changed");
}
