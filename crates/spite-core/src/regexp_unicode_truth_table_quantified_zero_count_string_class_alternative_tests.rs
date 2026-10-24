#[test]
fn truth_table_quantified_zero_count_alternatives_match_native_ordered_capture_tables() {
    use crate::RegExpUnicodeTruthTableQuantifiedZeroCountStringClassAlternativeMatcher as P;
    for (source, input, start, sticky, i, m, expected, captures) in [
        (
            "(((^)|(\\b))(([\\q{a|ab}]))*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0}",
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
            "(((^)|(\\b))(([\\q{a|ab}]))*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0}",
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
            "(((^)|(\\b))(([\\q{a|ab}]))*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0}",
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
            "(((^)|(\\b))(([\\q{a|ab}]))*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0}",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([\\q{|a|ab}])){1})*((\\b^)|($))){0,0}?",
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
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([\\q{|a|ab}])){1})*((\\b^)|($))){0,0}?",
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
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([\\q{|a|ab}])){1})*((\\b^)|($))){0,0}?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([\\q{|a|ab}])){1})*((\\b^)|($))){0,0}?",
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
            "(((^)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0000,0000}",
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
            "(((^)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0000,0000}",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0000,0000}",
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
            "(((^)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b^)|($))|((^)|(\\B))((){2,3})(($)|(\\B))){0000,0000}",
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
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)*((\\b^)|($))){0000}?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)*((\\b^)|($))){0000}?",
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
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)*((\\b^)|($))){0000}?",
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
            "(?:((^)|(\\B))((){2,3})(($)|(\\B))|((^)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)*((\\b^)|($))){0000}?",
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
            "((($)|(\\b))(([\\q{a|ab}]))+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0}",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\b))(([\\q{a|ab}]))+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0}",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\b))(([\\q{a|ab}]))+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0}",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\b))(([\\q{a|ab}]))+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0}",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([\\q{|a|ab}])){1})+?((\\B^)|(\\b$))){0,0}?",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([\\q{|a|ab}])){1})+?((\\B^)|(\\b$))){0,0}?",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([\\q{|a|ab}])){1})+?((\\B^)|(\\b$))){0,0}?",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([\\q{|a|ab}])){1})+?((\\B^)|(\\b$))){0,0}?",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0000,0000}",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "((($)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0000,0000}",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0000,0000}",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\b))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((\\B^)|(\\b$))|(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))){0000,0000}",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)+?((\\B^)|(\\b$))){0000}?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)+?((\\B^)|(\\b$))){0000}?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)+?((\\B^)|(\\b$))){0000}?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(((^)|(\\b))\\B)((){2,3})((\\b^)|(\\B$))|(($)|(\\b))((([a-z\\q{ab}]){1}){1,1}?)+?((\\B^)|(\\b$))){0000}?",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(((^)|($))(([\\q{a|ab}])){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0}",
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
            "(((^)|($))(([\\q{a|ab}])){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0}",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|($))(([\\q{a|ab}])){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0}",
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
            "(((^)|($))(([\\q{a|ab}])){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0}",
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
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([\\q{|a|ab}])){1}){2,3}((^)|(\\B))){0,0}?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([\\q{|a|ab}])){1}){2,3}((^)|(\\B))){0,0}?",
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
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([\\q{|a|ab}])){1}){2,3}((^)|(\\B))){0,0}?",
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
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([\\q{|a|ab}])){1}){2,3}((^)|(\\B))){0,0}?",
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
            "(((^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0000,0000}",
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
            "(((^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0000,0000}",
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
            "(((^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0000,0000}",
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
            "(((^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}((^)|(\\B))|(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))){0000,0000}",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([a-z\\q{ab}]){1}){1,1}?){2,3}((^)|(\\B))){0000}?",
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
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([a-z\\q{ab}]){1}){1,1}?){2,3}((^)|(\\B))){0000}?",
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
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([a-z\\q{ab}]){1}){1,1}?){2,3}((^)|(\\B))){0000}?",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\B))((){2,3})((\\b\\B)|(\\b\\B))|((^)|($))((([a-z\\q{ab}]){1}){1,1}?){2,3}((^)|(\\B))){0000}?",
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
            "(((\\b^)|($))(([\\q{a|ab}])){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0}",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))(([\\q{a|ab}])){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0}",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))(([\\q{a|ab}])){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0}",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))(([\\q{a|ab}])){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0}",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([\\q{|a|ab}])){1}){0002,0002}?(((^)|(\\b))\\B)){0,0}?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([\\q{|a|ab}])){1}){0002,0002}?(((^)|(\\b))\\B)){0,0}?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([\\q{|a|ab}])){1}){0002,0002}?(((^)|(\\b))\\B)){0,0}?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([\\q{|a|ab}])){1}){0002,0002}?(((^)|(\\b))\\B)){0,0}?",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0000,0000}",
            "ſa",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0000,0000}",
            "😀a",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0000,0000}",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b^)|($))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?(((^)|(\\b))\\B)|((\\b^)|(\\B$))((){2,3})((^)|(\\b))){0000,0000}",
            "",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?(((^)|(\\b))\\B)){0000}?",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?(((^)|(\\b))\\B)){0000}?",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?(((^)|(\\b))\\B)){0000}?",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\b^)|(\\B$))((){2,3})((^)|(\\b))|((\\b^)|($))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?(((^)|(\\b))\\B)){0000}?",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\B^)|(\\b$))(([\\q{a|ab}])){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0}",
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
            "(((\\B^)|(\\b$))(([\\q{a|ab}])){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0}",
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
            "(((\\B^)|(\\b$))(([\\q{a|ab}])){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0}",
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
            "(((\\B^)|(\\b$))(([\\q{a|ab}])){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0}",
            "ab\r\n",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([\\q{|a|ab}])){1}){0,}(($)|(\\B))){0,0}?",
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
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([\\q{|a|ab}])){1}){0,}(($)|(\\B))){0,0}?",
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
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([\\q{|a|ab}])){1}){0,}(($)|(\\B))){0,0}?",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([\\q{|a|ab}])){1}){0,}(($)|(\\B))){0,0}?",
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
            "(((\\B^)|(\\b$))(([\\p{Any}\\q{|ab}]){0001,0001}?){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0000,0000}",
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
            "(((\\B^)|(\\b$))(([\\p{Any}\\q{|ab}]){0001,0001}?){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0000,0000}",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((\\B^)|(\\b$))(([\\p{Any}\\q{|ab}]){0001,0001}?){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0000,0000}",
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
            "(((\\B^)|(\\b$))(([\\p{Any}\\q{|ab}]){0001,0001}?){0,}(($)|(\\B))|((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))){0000,0000}",
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
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([a-z\\q{ab}]){1}){1,1}?){0,}(($)|(\\B))){0000}?",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([a-z\\q{ab}]){1}){1,1}?){0,}(($)|(\\B))){0000}?",
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
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([a-z\\q{ab}]){1}){1,1}?){0,}(($)|(\\B))){0000}?",
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
            "(?:((\\b\\B)|(\\b\\B))((){2,3})(($)|(\\b))|((\\B^)|(\\b$))((([a-z\\q{ab}]){1}){1,1}?){0,}(($)|(\\B))){0000}?",
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
            "(((^)|(\\B))(([\\q{a|ab}])){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0}",
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
            "(((^)|(\\B))(([\\q{a|ab}])){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0}",
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
            "(((^)|(\\B))(([\\q{a|ab}])){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0}",
            "ab\r\n",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((^)|(\\B))(([\\q{a|ab}])){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0}",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([\\q{|a|ab}])){1}){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0,0}?",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([\\q{|a|ab}])){1}){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0,0}?",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([\\q{|a|ab}])){1}){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0,0}?",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([\\q{|a|ab}])){1}){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0,0}?",
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
            "(((^)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0000,0000}",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((^)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0000,0000}",
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
            "(((^)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0000,0000}",
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
            "(((^)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))|((^)|(\\b))((){2,3})((^)|($))){0000,0000}",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([a-z\\q{ab}]){1}){1,1}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0000}?",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([a-z\\q{ab}]){1}){1,1}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0000}?",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([a-z\\q{ab}]){1}){1,1}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0000}?",
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
            "(?:((^)|(\\b))((){2,3})((^)|($))|((^)|(\\B))((([a-z\\q{ab}]){1}){1,1}?){999999999999999999999999999,1000000000000000000000000000}?((\\b^)|(\\B$))){0000}?",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "((((^)|(\\b))\\B)(([\\q{a|ab}]))*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0}",
            "😀a",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\q{a|ab}]))*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0}",
            "ab\r\n",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "((((^)|(\\b))\\B)(([\\q{a|ab}]))*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0}",
            "",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\q{a|ab}]))*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0}",
            "a",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([\\q{|a|ab}])){1})*((\\b\\B)|(\\b\\B))){0,0}?",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([\\q{|a|ab}])){1})*((\\b\\B)|(\\b\\B))){0,0}?",
            "",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([\\q{|a|ab}])){1})*((\\b\\B)|(\\b\\B))){0,0}?",
            "a",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([\\q{|a|ab}])){1})*((\\b\\B)|(\\b\\B))){0,0}?",
            "ab",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0000,0000}",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0000,0000}",
            "a",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0000,0000}",
            "ab",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((((^)|(\\b))\\B)(([\\p{Any}\\q{|ab}]){0001,0001}?)*((\\b\\B)|(\\b\\B))|(($)|(\\b))((){2,3})((\\b^)|($))){0000,0000}",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([a-z\\q{ab}]){1}){1,1}?)*((\\b\\B)|(\\b\\B))){0000}?",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([a-z\\q{ab}]){1}){1,1}?)*((\\b\\B)|(\\b\\B))){0000}?",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([a-z\\q{ab}]){1}){1,1}?)*((\\b\\B)|(\\b\\B))){0000}?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:(($)|(\\b))((){2,3})((\\b^)|($))|(((^)|(\\b))\\B)((([a-z\\q{ab}]){1}){1,1}?)*((\\b\\B)|(\\b\\B))){0000}?",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "((($)|(\\B))(([\\q{a|ab}]))+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0}",
            "ab\r\n",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))(([\\q{a|ab}]))+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0}",
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
            "((($)|(\\B))(([\\q{a|ab}]))+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0}",
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
            "((($)|(\\B))(([\\q{a|ab}]))+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0}",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([\\q{|a|ab}])){1})+?((^)|(\\b))){0,0}?",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([\\q{|a|ab}])){1})+?((^)|(\\b))){0,0}?",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([\\q{|a|ab}])){1})+?((^)|(\\b))){0,0}?",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([\\q{|a|ab}])){1})+?((^)|(\\b))){0,0}?",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0000,0000}",
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
            "((($)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0000,0000}",
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
            "((($)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0000,0000}",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "((($)|(\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?)+?((^)|(\\b))|((^)|($))((){2,3})((\\B^)|(\\b$))){0000,0000}",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([a-z\\q{ab}]){1}){1,1}?)+?((^)|(\\b))){0000}?",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([a-z\\q{ab}]){1}){1,1}?)+?((^)|(\\b))){0000}?",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([a-z\\q{ab}]){1}){1,1}?)+?((^)|(\\b))){0000}?",
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
            "(?:((^)|($))((){2,3})((\\B^)|(\\b$))|(($)|(\\B))((([a-z\\q{ab}]){1}){1,1}?)+?((^)|(\\b))){0000}?",
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
            "(((\\b^)|(\\B$))(([\\q{a|ab}])){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0}",
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
            "(((\\b^)|(\\B$))(([\\q{a|ab}])){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0}",
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
            "(((\\b^)|(\\B$))(([\\q{a|ab}])){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0}",
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
            "(((\\b^)|(\\B$))(([\\q{a|ab}])){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0}",
            "!ab!",
            5,
            true,
            true,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([\\q{|a|ab}])){1}){2,3}(($)|(\\b))){0,0}?",
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
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([\\q{|a|ab}])){1}){2,3}(($)|(\\b))){0,0}?",
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
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([\\q{|a|ab}])){1}){2,3}(($)|(\\b))){0,0}?",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([\\q{|a|ab}])){1}){2,3}(($)|(\\b))){0,0}?",
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
            "(((\\b^)|(\\B$))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0000,0000}",
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
            "(((\\b^)|(\\B$))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0000,0000}",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b^)|(\\B$))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0000,0000}",
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
            "(((\\b^)|(\\B$))(([\\p{Any}\\q{|ab}]){0001,0001}?){2,3}(($)|(\\b))|((\\b^)|($))((){2,3})((^)|(\\B))){0000,0000}",
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
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([a-z\\q{ab}]){1}){1,1}?){2,3}(($)|(\\b))){0000}?",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([a-z\\q{ab}]){1}){1,1}?){2,3}(($)|(\\b))){0000}?",
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
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([a-z\\q{ab}]){1}){1,1}?){2,3}(($)|(\\b))){0000}?",
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
            "(?:((\\b^)|($))((){2,3})((^)|(\\B))|((\\b^)|(\\B$))((([a-z\\q{ab}]){1}){1,1}?){2,3}(($)|(\\b))){0000}?",
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
            "(((\\b\\B)|(\\b\\B))(([\\q{a|ab}])){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0}",
            "a",
            1,
            false,
            false,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\q{a|ab}])){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0}",
            "ab",
            0,
            true,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\q{a|ab}])){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0}",
            "!ab!",
            5,
            true,
            false,
            true,
            None,
            vec![],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\q{a|ab}])){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0}",
            "a\n!",
            0,
            false,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([\\q{|a|ab}])){1}){0002,0002}?((^)|($))){0,0}?",
            "ab",
            0,
            true,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([\\q{|a|ab}])){1}){0002,0002}?((^)|($))){0,0}?",
            "!ab!",
            5,
            true,
            true,
            false,
            None,
            vec![],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([\\q{|a|ab}])){1}){0002,0002}?((^)|($))){0,0}?",
            "a\n!",
            0,
            false,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([\\q{|a|ab}])){1}){0002,0002}?((^)|($))){0,0}?",
            "ſa",
            1,
            false,
            true,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0000,0000}",
            "!ab!",
            5,
            true,
            false,
            false,
            None,
            vec![],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0000,0000}",
            "a\n!",
            0,
            false,
            true,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0000,0000}",
            "ſa",
            1,
            false,
            false,
            true,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(((\\b\\B)|(\\b\\B))(([\\p{Any}\\q{|ab}]){0001,0001}?){0002,0002}?((^)|($))|((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)){0000,0000}",
            "😀a",
            0,
            true,
            true,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?((^)|($))){0000}?",
            "a\n!",
            0,
            false,
            false,
            false,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?((^)|($))){0000}?",
            "ſa",
            1,
            false,
            true,
            false,
            Some(1..1),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?((^)|($))){0000}?",
            "😀a",
            0,
            true,
            false,
            true,
            Some(0..0),
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ],
        ),
        (
            "(?:((\\B^)|(\\b$))((){2,3})(((^)|(\\b))\\B)|((\\b\\B)|(\\b\\B))((([a-z\\q{ab}]){1}){1,1}?){0002,0002}?((^)|($))){0000}?",
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
fn truth_table_quantified_zero_count_alternatives_remain_flat_and_keep_every_slot_undefined() {
    use crate::RegExpUnicodeTruthTableQuantifiedZeroCountStringClassAlternativeMatcher as P;
    let source = format!(
        r"({}([\q{{a|ab}}]){}|()){{0}}",
        "(".repeat(20_000),
        "){2,}?".repeat(20_000)
    );
    let p = P::compile_with_work(&JsString::from(source.as_str()), false, false, |_| {
        Ok::<(), ()>(())
    })
    .unwrap()
    .unwrap();
    assert_eq!(p.capture_count(), 20_003);
    for input in ["", "ab", "😀ab"] {
        let input = JsString::from(input);
        for requested in 0..=input.len() + 1 {
            for sticky in [false, true] {
                let mut work = 0;
                let found = p
                    .find_with_captures_and_work(&input, requested, sticky, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    })
                    .unwrap();
                let normalized =
                    if requested == 1 && input.len() > 1 && input.code_units()[0] == 0xd83d {
                        0
                    } else {
                        requested
                    };
                assert_eq!(
                    found.as_ref().map(|m| m.repeated.range.clone()),
                    (normalized <= input.len()).then_some(normalized..normalized)
                );
                assert!(work <= 32);
                if let Some(found) = found {
                    assert!(!found.repeated.class_selected);
                    assert_eq!(found.repeated.last_iteration, None);
                    for slot in 0..=p.capture_count() {
                        assert_eq!(p.capture_range(slot, &found), None);
                    }
                }
            }
        }
    }
    let clone = p.clone();
    drop(p);
    assert_eq!(clone.capture_count(), 20_003);
    drop(clone);
    let source =
        JsString::from(format!(r"(([\q{{a|ab}}])+?|()){{{}}}?", "0".repeat(1000)).as_str());
    let p = P::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(p.capture_count(), 3);
    let f = p
        .find_with_captures_and_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(f.repeated.range, 0..0);
    assert!((0..=p.capture_count()).all(|n| p.capture_range(n, &f).is_none()));

    let word = "x".repeat(20_000);
    let source = JsString::from(
        format!(r"((([\p{{Any}}\q{{|{word}|{word}x}}]){{2,3}}?)?|()){{0000,0000}}?").as_str(),
    );
    let p = P::compile_with_work(&source, true, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let input = JsString::from(word.as_str());
    let mut work = 0;
    let found = p
        .find_with_captures_and_work(&input, 0, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(found.repeated.range, 0..0);
    assert!(work <= 32);
    assert!((0..=p.capture_count()).all(|n| p.capture_range(n, &found).is_none()));
}
#[test]
fn truth_table_quantified_zero_count_alternatives_validate_all_terms_and_recover_work_aborts() {
    use crate::RegExpUnicodeTruthTableQuantifiedZeroCountStringClassAlternativeMatcher as P;
    let source = JsString::from(
        r"((((^)|(\b))(([\q{|a|ab}])+?){2,3}?((\B)|($)))?|()) {0}"
            .replace(" ", "")
            .as_str(),
    );
    let mut calls = 0;
    let p = P::compile_with_work(&source, false, false, |_| {
        calls += 1;
        Ok::<(), ()>(())
    })
    .unwrap()
    .unwrap();
    for abort in 0..calls {
        let mut call = 0;
        assert!(matches!(
            P::compile_with_work(&source, false, false, |_| {
                let fail = call == abort;
                call += 1;
                if fail { Err(37) } else { Ok(()) }
            }),
            Err(crate::RegExpUnicodeSetCompileError::Work(37))
        ));
    }
    let input = JsString::from("😀ab");
    let mut calls = 0;
    p.find_with_captures_and_work(&input, 1, true, |_| {
        calls += 1;
        Ok::<(), ()>(())
    })
    .unwrap();
    for abort in 0..calls {
        let mut call = 0;
        assert!(matches!(
            p.find_with_captures_and_work(&input, 1, true, |_| {
                let fail = call == abort;
                call += 1;
                if fail { Err(41) } else { Ok(()) }
            }),
            Err(crate::RegExpUnicodeMatchError::Work(41))
        ));
    }
    assert_eq!(
        p.find_with_captures_and_work(&input, 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap()
            .repeated
            .range,
        0..0
    );
    for source in [
        r"((([\q{ab}&&\p{RGI_Emoji}]){2})?|()){0}",
        r"((([\q{a|ab}]){2})?|z){0}",
        r"((([\q{a|ab}]){2})?|\1){0}",
        r"((([\q{a|ab}]){2})?|([a])){0}",
        r"((([\q{a|ab}])+)?|()){0,1}",
        r"((([\q{a|ab}])?)?|()){0}",
        r"^((([\q{a|ab}])?)?|()){0}",
        r"((([\q{a|ab}])?)?|()){0}$",
        r"((([\q{a|ab}])?)?|()){0,1}",
        r"(([\q{a|ab}])|()){0}",
    ] {
        assert!(
            P::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
