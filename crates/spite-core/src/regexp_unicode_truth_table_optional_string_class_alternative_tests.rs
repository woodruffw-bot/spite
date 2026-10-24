#[test]
fn truth_table_optional_alternatives_match_native_ordered_capture_tables() {
    use crate::RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher as P;
    for (source, input, start, sticky, i, m, expected, captures) in [
        (
            "(((^)|(\\b))([\\q{a|ab}])((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B)))?",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\b))([\\q{a|ab}])((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B)))?",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\b))([\\q{a|ab}])((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B)))?",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..2),
            vec![
                Some(0..2),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..2),
                Some(2..2),
                None,
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((^)|(\\b))([\\q{a|ab}])((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B)))?",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([\\q{|a|ab}])){1}((\\b^)|($)))??",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([\\q{|a|ab}])){1}((\\b^)|($)))??",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([\\q{|a|ab}])){1}((\\b^)|($)))??",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([\\q{|a|ab}])){1}((\\b^)|($)))??",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0,1}",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..2),
            vec![
                Some(0..2),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..2),
                Some(2..2),
                None,
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((^)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0,1}",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0,1}",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..1),
            vec![
                Some(0..1),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..1),
                Some(1..1),
                None,
                Some(1..1),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((^)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0,1}",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|($))){0000,0001}?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|($))){0000,0001}?",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|($))){0000,0001}?",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|($))){0000,0001}?",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "((($)|(\\b))([\\q{a|ab}])((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$)))?",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((($)|(\\b))([\\q{a|ab}])((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$)))?",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..2),
            vec![
                Some(0..2),
                Some(0..0),
                None,
                Some(0..0),
                Some(0..2),
                Some(2..2),
                None,
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "((($)|(\\b))([\\q{a|ab}])((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$)))?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\b))([\\q{a|ab}])((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$)))?",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..1),
            vec![
                Some(0..1),
                Some(0..0),
                None,
                Some(0..0),
                Some(0..1),
                Some(1..1),
                None,
                Some(1..1),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([\\q{|a|ab}])){1}((\\B^)|(\\b$)))??",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([\\q{|a|ab}])){1}((\\B^)|(\\b$)))??",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([\\q{|a|ab}])){1}((\\B^)|(\\b$)))??",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([\\q{|a|ab}])){1}((\\B^)|(\\b$)))??",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((($)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0,1}",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "((($)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0,1}",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((($)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0,1}",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..2),
            vec![
                Some(1..2),
                Some(1..1),
                None,
                Some(1..1),
                Some(1..2),
                Some(2..2),
                None,
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "((($)|(\\b))([\\p{Any}\\q{|ab}]){0001,0001}?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0,1}",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\B^)|(\\b$))){0000,0001}?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\B^)|(\\b$))){0000,0001}?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\B^)|(\\b$))){0000,0001}?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))(([a-z\\q{ab}]){1}){1,1}?((\\B^)|(\\b$))){0000,0001}?",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(((^)|($))([\\q{a|ab}])((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B)))?",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..1),
            vec![
                Some(0..1),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..1),
                Some(1..1),
                None,
                Some(1..1),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((^)|($))([\\q{a|ab}])((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B)))?",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|($))([\\q{a|ab}])((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B)))?",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|($))([\\q{a|ab}])((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B)))?",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([\\q{|a|ab}])){1}((^)|(\\B)))??",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([\\q{|a|ab}])){1}((^)|(\\B)))??",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([\\q{|a|ab}])){1}((^)|(\\B)))??",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([\\q{|a|ab}])){1}((^)|(\\B)))??",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0,1}",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0,1}",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0,1}",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0,1}",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\B))){0000,0001}?",
            "ſa",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\B))){0000,0001}?",
            "😀a",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\B))){0000,0001}?",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\B))){0000,0001}?",
            "",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b^)|($))([\\q{a|ab}])(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b)))?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))([\\q{a|ab}])(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b)))?",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))([\\q{a|ab}])(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b)))?",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))([\\q{a|ab}])(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b)))?",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([\\q{|a|ab}])){1}(((^)|(\\b))\\B))??",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([\\q{|a|ab}])){1}(((^)|(\\b))\\B))??",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([\\q{|a|ab}])){1}(((^)|(\\b))\\B))??",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([\\q{|a|ab}])){1}(((^)|(\\b))\\B))??",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0,1}",
            "ſa",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0,1}",
            "😀a",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0,1}",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))([\\p{Any}\\q{|ab}]){0001,0001}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0,1}",
            "",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([a-z\\q{ab}]){1}){1,1}?(((^)|(\\b))\\B)){0000,0001}?",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([a-z\\q{ab}]){1}){1,1}?(((^)|(\\b))\\B)){0000,0001}?",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([a-z\\q{ab}]){1}){1,1}?(((^)|(\\b))\\B)){0000,0001}?",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))(([a-z\\q{ab}]){1}){1,1}?(((^)|(\\b))\\B)){0000,0001}?",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\q{a|ab}])(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b)))?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\q{a|ab}])(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b)))?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\q{a|ab}])(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b)))?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\q{a|ab}])(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b)))?",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([\\q{|a|ab}])){1}(($)|(\\B)))??",
            "ſa",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([\\q{|a|ab}])){1}(($)|(\\B)))??",
            "😀a",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([\\q{|a|ab}])){1}(($)|(\\B)))??",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([\\q{|a|ab}])){1}(($)|(\\B)))??",
            "",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0,1}",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0,1}",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((\\B^)|(\\b$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0,1}",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0,1}",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\B))){0000,0001}?",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\B))){0000,0001}?",
            "",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\B))){0000,0001}?",
            "a",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\B))){0000,0001}?",
            "ab",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\q{a|ab}])((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($)))?",
            "ſa",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\q{a|ab}])((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($)))?",
            "😀a",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\q{a|ab}])((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($)))?",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((^)|(\\B))([\\q{a|ab}])((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($)))?",
            "",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([\\q{|a|ab}])){1}((\\b^)|(\\B$)))??",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([\\q{|a|ab}])){1}((\\b^)|(\\B$)))??",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([\\q{|a|ab}])){1}((\\b^)|(\\B$)))??",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([\\q{|a|ab}])){1}((\\b^)|(\\B$)))??",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0,1}",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0,1}",
            "",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0,1}",
            "a",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((^)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0,1}",
            "ab",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|(\\B$))){0000,0001}?",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|(\\B$))){0000,0001}?",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|(\\B$))){0000,0001}?",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((\\b^)|(\\B$))){0000,0001}?",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "((((^)|(\\b))\\B)([\\q{a|ab}])((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($)))?",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\q{a|ab}])((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($)))?",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "((((^)|(\\b))\\B)([\\q{a|ab}])((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($)))?",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\q{a|ab}])((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($)))?",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([\\q{|a|ab}])){1}((\\b\\B)|(\\b\\B)))??",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([\\q{|a|ab}])){1}((\\b\\B)|(\\b\\B)))??",
            "",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([\\q{|a|ab}])){1}((\\b\\B)|(\\b\\B)))??",
            "a",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([\\q{|a|ab}])){1}((\\b\\B)|(\\b\\B)))??",
            "ab",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\p{Any}\\q{|ab}]){0001,0001}?((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0,1}",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\p{Any}\\q{|ab}]){0001,0001}?((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0,1}",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\p{Any}\\q{|ab}]){0001,0001}?((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0,1}",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)([\\p{Any}\\q{|ab}]){0001,0001}?((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0,1}",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([a-z\\q{ab}]){1}){1,1}?((\\b\\B)|(\\b\\B))){0000,0001}?",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([a-z\\q{ab}]){1}){1,1}?((\\b\\B)|(\\b\\B))){0000,0001}?",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([a-z\\q{ab}]){1}){1,1}?((\\b\\B)|(\\b\\B))){0000,0001}?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)(([a-z\\q{ab}]){1}){1,1}?((\\b\\B)|(\\b\\B))){0000,0001}?",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "((($)|(\\B))([\\q{a|ab}])((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$)))?",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))([\\q{a|ab}])((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$)))?",
            "",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "((($)|(\\B))([\\q{a|ab}])((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$)))?",
            "a",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "((($)|(\\B))([\\q{a|ab}])((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$)))?",
            "ab",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([\\q{|a|ab}])){1}((^)|(\\b)))??",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([\\q{|a|ab}])){1}((^)|(\\b)))??",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([\\q{|a|ab}])){1}((^)|(\\b)))??",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([\\q{|a|ab}])){1}((^)|(\\b)))??",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0,1}",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "((($)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0,1}",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "((($)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0,1}",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0,1}",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\b))){0000,0001}?",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\b))){0000,0001}?",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\b))){0000,0001}?",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|(\\b))){0000,0001}?",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\q{a|ab}])(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B)))?",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\q{a|ab}])(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B)))?",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\q{a|ab}])(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B)))?",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..2),
            vec![
                Some(0..2),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..2),
                Some(2..2),
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\q{a|ab}])(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B)))?",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([\\q{|a|ab}])){1}(($)|(\\b)))??",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([\\q{|a|ab}])){1}(($)|(\\b)))??",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([\\q{|a|ab}])){1}(($)|(\\b)))??",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([\\q{|a|ab}])){1}(($)|(\\b)))??",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0,1}",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..2),
            vec![
                Some(0..2),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..2),
                Some(2..2),
                Some(2..2),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0,1}",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b^)|(\\B$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0,1}",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..1),
            vec![
                Some(0..1),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..1),
                Some(1..1),
                Some(1..1),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            "(((\\b^)|(\\B$))([\\p{Any}\\q{|ab}]){0001,0001}?(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0,1}",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\b))){0000,0001}?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\b))){0000,0001}?",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\b))){0000,0001}?",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))(([a-z\\q{ab}]){1}){1,1}?(($)|(\\b))){0000,0001}?",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\q{a|ab}])((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B))?",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\q{a|ab}])((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B))?",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\q{a|ab}])((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B))?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\q{a|ab}])((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B))?",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([\\q{|a|ab}])){1}((^)|($)))??",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([\\q{|a|ab}])){1}((^)|($)))??",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([\\q{|a|ab}])){1}((^)|($)))??",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([\\q{|a|ab}])){1}((^)|($)))??",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0,1}",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0,1}",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0,1}",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))([\\p{Any}\\q{|ab}]){0001,0001}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0,1}",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|($))){0000,0001}?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|($))){0000,0001}?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|($))){0000,0001}?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))(([a-z\\q{ab}]){1}){1,1}?((^)|($))){0000,0001}?",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
    ] {
        let p = P::compile_with_work(&JsString::from(source), i, m, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap_or_else(|| panic!("declined {source}"));
        let f =
            p.find_with_captures_and_work(&JsString::from(input), start, sticky, |_| {
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(
            f.as_ref().map(|f| f.repeated.range.clone()),
            expected,
            "{source}, {input:?}, {start}, {sticky}, {i}, {m}"
        );
        if let Some(f) = f {
            assert_eq!(
                (0..p.capture_count())
                    .map(|n| p.capture_range(n, &f))
                    .collect::<Vec<_>>(),
                captures,
                "{source}, {input:?}, {start}, {sticky}, {i}, {m}"
            );
            assert_eq!(p.capture_range(p.capture_count(), &f), None);
        }
    }
}

#[test]
fn truth_table_optional_alternatives_match_independent_progress_and_capture_oracle() {
    use crate::RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher as P;
    let profiles = [
        ("^", r"\b", 1u8, 4u8),
        ("$", r"\b", 2, 4),
        ("^", "$", 1, 2),
        (r"\b^", "$", 5, 2),
        (r"\B^", r"\b$", 9, 6),
        ("^", r"\B", 1, 8),
        ("$", r"\B", 2, 8),
        (r"\b\B", r"\b\B", 12, 12),
    ];
    for (k, (left, right, lf, rf)) in profiles.into_iter().enumerate() {
        let (al, ar, alf, arf) = profiles[(k + 3) % profiles.len()];
        let (el, er, _, _) = profiles[(k + 5) % profiles.len()];
        let (tl, tr, _, _) = profiles[(k + 7) % profiles.len()];
        for (class, scalar, never) in [
            (r"[\q{a|ab}]", false, false),
            (r"[\q{|a|ab}]", false, false),
            (r"[\p{Any}\q{|ab}]", true, false),
            (r"[\q{ab}&&\q{cd}]", false, true),
        ] {
            for class_first in [false, true] {
                for body_count in ["", "{1}", "{0001,0001}?"] {
                    for outside in ["", "{2,3}?", "{999999999999999999999999}", "?", "{0,2}?"] {
                        for (q, greedy) in [
                            ("?", true),
                            ("??", false),
                            ("{0,1}", true),
                            ("{0000,0001}?", false),
                        ] {
                            let a = format!(
                                "(({left})|({right})){outside}({class}){body_count}(({al})|({ar})){outside}"
                            );
                            let b = format!("(({el})|({er})){outside}()(({tl})|({tr})){outside}");
                            let source = if class_first {
                                format!("({a}|{b}){q}")
                            } else {
                                format!("({b}|{a}){q}")
                            };
                            for (i, m) in
                                [(false, false), (true, false), (false, true), (true, true)]
                            {
                                let plan = P::compile_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap_or_else(|| panic!("declined {source}"));
                                assert_eq!(plan.capture_count(), 15);
                                for text in [
                                    "",
                                    "a",
                                    "ab",
                                    "aba",
                                    "!ab!",
                                    "AB",
                                    "ſab!",
                                    "😀a",
                                    "a\n",
                                    "\u{2028}ab\u{2029}",
                                ] {
                                    let input = JsString::from(text);
                                    let u = input.code_units();
                                    let skip = matches!(outside, "?" | "{0,2}?");
                                    let predicate = |at: usize, flags: u8| {
                                        let word = |p: Option<&u16>| {
                                            p.is_some_and(|p|matches!(*p,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)||(i&&matches!(*p,0x17f|0x212a)))
                                        };
                                        let boundary =
                                            word(at.checked_sub(1).and_then(|p| u.get(p)))
                                                != word(u.get(at));
                                        (flags & 1 == 0
                                            || at == 0
                                            || (m
                                                && matches!(
                                                    u[at - 1],
                                                    0xa | 0xd | 0x2028 | 0x2029
                                                )))
                                            && (flags & 2 == 0
                                                || at == u.len()
                                                || (m
                                                    && matches!(
                                                        u[at],
                                                        0xa | 0xd | 0x2028 | 0x2029
                                                    )))
                                            && (flags & 4 == 0 || boundary)
                                            && (flags & 8 == 0 || !boundary)
                                    };
                                    for requested in 0..=u.len() + 1 {
                                        for sticky in [false, true] {
                                            let mut at = requested;
                                            if at > 0
                                                && at < u.len()
                                                && (0xdc00..=0xdfff).contains(&u[at])
                                                && (0xd800..=0xdbff).contains(&u[at - 1])
                                            {
                                                at -= 1;
                                            }
                                            let mut expected = None;
                                            if at <= u.len() {
                                                let mut end = at;
                                                if greedy
                                                    && !never
                                                    && (skip
                                                        || predicate(at, lf)
                                                        || predicate(at, rf))
                                                {
                                                    let fold = |p: Option<&u16>| {
                                                        p.copied().map(|p| {
                                                            if i && (0x41..=0x5a).contains(&p) {
                                                                p + 32
                                                            } else {
                                                                p
                                                            }
                                                        })
                                                    };
                                                    let astral = u.get(at).is_some_and(|p| {
                                                        (0xd800..=0xdbff).contains(p)
                                                    }) && u.get(at + 1).is_some_and(
                                                        |p| (0xdc00..=0xdfff).contains(p),
                                                    );
                                                    for width in [2, 1] {
                                                        let member = if width == 2 {
                                                            (fold(u.get(at)) == Some(0x61)
                                                                && fold(u.get(at + 1))
                                                                    == Some(0x62))
                                                                || (scalar && astral)
                                                        } else {
                                                            fold(u.get(at)) == Some(0x61)
                                                                || (scalar
                                                                    && at < u.len()
                                                                    && !astral)
                                                        };
                                                        if member
                                                            && (skip
                                                                || predicate(at + width, alf)
                                                                || predicate(at + width, arf))
                                                        {
                                                            end = at + width;
                                                            break;
                                                        }
                                                    }
                                                }
                                                expected = Some(at..end);
                                            }
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    requested,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            assert_eq!(
                                                found.as_ref().map(|f| f.repeated.range.clone()),
                                                expected,
                                                "{source}, {text:?}, {requested}, {sticky}, {i}, {m}"
                                            );
                                            if let Some(f) = found {
                                                let range = f.repeated.range.clone();
                                                let selected = range.start != range.end;
                                                assert_eq!(f.repeated.class_selected, selected);
                                                assert_eq!(
                                                    f.repeated.last_iteration,
                                                    selected.then(|| range.clone())
                                                );
                                                let mut want = vec![None; 15];
                                                if selected {
                                                    want[0] = Some(range.clone());
                                                    let offset = if class_first { 1 } else { 8 };
                                                    want[offset + 3] = Some(range.clone());
                                                    if !skip {
                                                        want[offset] =
                                                            Some(range.start..range.start);
                                                        want[offset
                                                            + if predicate(range.start, lf) {
                                                                1
                                                            } else {
                                                                2
                                                            }] = Some(range.start..range.start);
                                                        want[offset + 4] =
                                                            Some(range.end..range.end);
                                                        want[offset
                                                            + if predicate(range.end, alf) {
                                                                5
                                                            } else {
                                                                6
                                                            }] = Some(range.end..range.end);
                                                    }
                                                }
                                                assert_eq!(
                                                    (0..15)
                                                        .map(|n| plan.capture_range(n, &f))
                                                        .collect::<Vec<_>>(),
                                                    want,
                                                    "{source}, {text:?}, {requested}"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn truth_table_optional_alternatives_reject_empty_iterations_and_keep_branch_capture_rollback() {
    use crate::RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher as P;
    for (source, text, want, selected) in [
        (r"(()|([\q{|a|ab}]))?", "ab", 0..2, true),
        (r"(()|([\q{|a|ab}]))??", "ab", 0..0, false),
        (r"(([\q{}])|()){0,1}", "", 0..0, false),
        (
            r"(((^)|(\b))([\q{ab!|a}])((^)|(\B))|())?",
            "ab!Z",
            0..1,
            true,
        ),
        (r"(((\b\B)|(\b\B))()|([\q{a|ab}]))?", "!ab", 0..0, false),
    ] {
        let p = P::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let f = p
            .find_with_captures_and_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(f.repeated.range, want, "{source}");
        assert_eq!(f.repeated.class_selected, selected, "{source}");
        assert_eq!(p.capture_range(0, &f), selected.then_some(want));
        if !selected {
            assert!((0..p.capture_count()).all(|n| p.capture_range(n, &f).is_none()));
        }
    }
    let source = JsString::from(
        format!(
            r"({}((^)|(\b)){}([\q{{a|ab}}])|())?",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let p = P::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let f = p
        .find_with_captures_and_work(&JsString::from("ab"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(p.capture_count(), 20_006);
    assert_eq!(p.capture_range(0, &f), Some(0..2));
    assert_eq!(p.capture_range(1, &f), Some(0..0));
    assert_eq!(p.capture_range(20_004, &f), Some(0..2));
    assert_eq!(p.capture_range(20_005, &f), None);
    let clone = p.clone();
    drop(p);
    assert_eq!(clone.capture_range(1, &f), Some(0..0));
    assert_eq!(clone.capture_range(usize::MAX, &f), None);
    let word = "a".repeat(20_000);
    let source = JsString::from(
        format!(r"(()|((^)|(\b))([\p{{Any}}\q{{|{word}|{word}x}}])){{0000,0001}}").as_str(),
    );
    let p = P::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let f = p
        .find_with_captures_and_work(
            &JsString::from(word.as_str()),
            0,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(f.repeated.range, 0..20_000);
    assert!(f.repeated.class_selected);
    assert_eq!(p.capture_range(0, &f), Some(0..20_000));
}
#[test]
fn truth_table_optional_alternatives_charge_cancellation_and_validate_even_lazy_never_branches() {
    use crate::{
        RegExpUnicodeMatchError, RegExpUnicodeSetCompileError,
        RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher as P,
    };
    for source in [
        r"((([\q{|a|ab}]))|())?",
        r"(()|((^)|(\b))([\p{Any}\q{|ab}])){0,1}?",
        r"(((\b\B)|(\b\B))([\q{ab}&&\q{cd}])|())?",
    ] {
        let source = JsString::from(source);
        let mut calls = 0;
        let plan = P::compile_with_work(&source, false, true, |_| {
            calls += 1;
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for stop in 0..calls {
            let mut n = 0;
            let result = P::compile_with_work(&source, false, true, |_| {
                let call = n;
                n += 1;
                if call == stop { Err(call) } else { Ok(()) }
            });
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(n))if n==stop));
        }
        for text in ["ab!\n", "", "😀a"] {
            let input = JsString::from(text);
            let mut calls = 0;
            let expected = plan
                .find_with_captures_and_work(&input, 0, false, |_| {
                    calls += 1;
                    Ok::<(), usize>(())
                })
                .unwrap();
            for stop in 0..calls {
                let mut n = 0;
                let result = plan.find_with_captures_and_work(&input, 0, false, |_| {
                    let call = n;
                    n += 1;
                    if call == stop { Err(call) } else { Ok(()) }
                });
                assert!(matches!(result,Err(RegExpUnicodeMatchError::Work(n))if n==stop));
                assert_eq!(
                    plan.find_with_captures_and_work(&input, 0, false, |_| Ok::<(), usize>(()))
                        .unwrap(),
                    expected
                );
            }
        }
    }
    for source in [
        r"(((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])|())??",
        r"(((^)|(\b))([\q{a|ab}])|z)??",
        r"(((^)|(\b))([\q{a|ab}])|\1)?",
        r"(((^)|(\b))([\q{a|ab}])|([a]))?",
        r"(((^)|(\b))([\q{a|ab}])|())",
        r"(((^)|(\b))([\q{a|ab}])|()){0}",
        r"(((^)|(\b))([\q{a|ab}])|()){0,2}",
        r"(((^)|(\b))([\q{a|ab}])|()){1,1}",
        r"^(((^)|(\b))([\q{a|ab}])|())?",
        r"(((^)|(\b))([\q{a|ab}])|())?$",
        r"((((^)|(\b))([\q{a|ab}])|())?)",
    ] {
        assert!(
            P::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
