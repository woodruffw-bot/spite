//! Grapheme_Extend and its exact Gr_Ext alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unicode_grapheme_extend_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Grapheme_Extend}",
        r"\P{Gr_Ext}",
        r"[^\p{Gr_Ext}]",
        r"[\p{Gr_Ext}K]",
        r"[\P{Grapheme_Extend}K]",
        r"^(?<x>[\p{Gr_Ext}K])$",
        r"(?<x>\P{Grapheme_Extend})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{2ff}\u{300}\u{345}\u{36f}\u{370}\u{399}\u{3b9}\u{482}\u{483}\u{489}\u{48a}\u{590}\u{591}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5ca}\u{60f}\u{610}\u{61a}\u{61b}\u{64a}\u{64b}\u{65f}\u{660}\u{66f}\u{670}\u{671}\u{6d5}\u{6d6}\u{6dc}\u{6dd}\u{6de}\u{6df}\u{6e4}\u{6e5}\u{6e6}\u{6e7}\u{6e8}\u{6e9}\u{6ea}\u{6ed}\u{6ee}\u{710}\u{711}\u{712}\u{72f}\u{730}\u{74a}\u{74b}\u{7a5}\u{7a6}\u{7b0}\u{7b1}\u{7ea}\u{7eb}\u{7f3}\u{7f4}\u{7fc}\u{7fd}\u{7fe}\u{815}\u{816}\u{819}\u{81a}\u{81b}\u{823}\u{824}\u{825}\u{827}\u{828}\u{829}\u{82d}\u{82e}\u{858}\u{859}\u{85b}\u{85c}\u{896}\u{897}\u{89f}\u{8a0}\u{8c9}\u{8ca}\u{8e1}\u{8e2}\u{8e3}\u{902}\u{903}\u{939}\u{93a}\u{93b}\u{93c}\u{93d}\u{940}\u{941}\u{948}\u{949}\u{94c}\u{94d}\u{94e}\u{950}\u{951}\u{957}\u{958}\u{961}\u{962}\u{963}\u{964}\u{980}\u{981}\u{982}\u{9bb}\u{9bc}\u{9bd}\u{9be}\u{9bf}\u{9c0}\u{9c1}\u{9c4}\u{9c5}\u{9cc}\u{9cd}\u{9ce}\u{9d6}\u{9d7}\u{9d8}\u{9e1}\u{9e2}\u{9e3}\u{9e4}\u{9fd}\u{9fe}\u{9ff}\u{a00}\u{a01}\u{a02}\u{a03}\u{a3b}\u{a3c}\u{a3d}\u{a40}\u{a41}\u{a42}\u{a43}\u{a46}\u{a47}\u{a48}\u{a49}\u{a4a}\u{a4b}\u{a4d}\u{a4e}\u{a50}\u{a51}\u{a52}\u{a6f}\u{a70}\u{a71}\u{a72}\u{a74}\u{a75}\u{a76}\u{a80}\u{a81}\u{a82}\u{a83}\u{abb}\u{abc}\u{abd}\u{ac0}\u{ac1}\u{ac5}\u{ac6}\u{ac7}\u{ac8}\u{ac9}\u{acc}\u{acd}\u{ace}\u{ae1}\u{ae2}\u{ae3}\u{ae4}\u{af9}\u{afa}\u{aff}\u{b00}\u{b01}\u{b02}\u{b3b}\u{b3c}\u{b3d}\u{b3e}\u{b3f}\u{b40}\u{b41}\u{b44}\u{b45}\u{b4c}\u{b4d}\u{b4e}\u{b52}\u{b57}\u{b58}\u{b61}\u{b62}\u{b63}\u{b64}\u{b81}\u{b82}\u{b83}\u{bbd}\u{bbe}\u{bbf}\u{bc0}\u{bc1}\u{bcc}\u{bcd}\u{bce}\u{bd6}\u{bd7}\u{bd8}\u{bff}\u{c00}\u{c01}\u{c03}\u{c04}\u{c05}\u{c3b}\u{c3c}\u{c3d}\u{c3e}\u{c40}\u{c41}\u{c45}\u{c46}\u{c48}\u{c49}\u{c4a}\u{c4d}\u{c4e}\u{c54}\u{c55}\u{c56}\u{c57}\u{c61}\u{c62}\u{c63}\u{c64}\u{c80}\u{c81}\u{c82}\u{cbb}\u{cbc}\u{cbd}\u{cbe}\u{cbf}\u{cc0}\u{cc1}\u{cc2}\u{cc3}\u{cc5}\u{cc6}\u{cc8}\u{cc9}\u{cca}\u{ccd}\u{cce}\u{cd4}\u{cd5}\u{cd6}\u{cd7}\u{ce1}\u{ce2}\u{ce3}\u{ce4}\u{cff}\u{d00}\u{d01}\u{d02}\u{d3a}\u{d3b}\u{d3c}\u{d3d}\u{d3e}\u{d3f}\u{d40}\u{d41}\u{d44}\u{d45}\u{d4c}\u{d4d}\u{d4e}\u{d56}\u{d57}\u{d58}\u{d61}\u{d62}\u{d63}\u{d64}\u{d80}\u{d81}\u{d82}\u{dc9}\u{dca}\u{dcb}\u{dce}\u{dcf}\u{dd0}\u{dd1}\u{dd2}\u{dd4}\u{dd5}\u{dd6}\u{dd7}\u{dde}\u{ddf}\u{de0}\u{e30}\u{e31}\u{e32}\u{e33}\u{e34}\u{e3a}\u{e3b}\u{e46}\u{e47}\u{e4e}\u{e4f}\u{eb0}\u{eb1}\u{eb2}\u{eb3}\u{eb4}\u{ebc}\u{ebd}\u{ec7}\u{ec8}\u{ece}\u{ecf}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f70}\u{f71}\u{f7e}\u{f7f}\u{f80}\u{f84}\u{f85}\u{f86}\u{f87}\u{f88}\u{f8c}\u{f8d}\u{f97}\u{f98}\u{f99}\u{fbc}\u{fbd}\u{fc5}\u{fc6}\u{fc7}\u{102c}\u{102d}\u{1030}\u{1031}\u{1032}\u{1037}\u{1038}\u{1039}\u{103a}\u{103b}\u{103c}\u{103d}\u{103e}\u{103f}\u{1057}\u{1058}\u{1059}\u{105a}\u{105d}\u{105e}\u{1060}\u{1061}\u{1070}\u{1071}\u{1074}\u{1075}\u{1081}\u{1082}\u{1083}\u{1084}\u{1085}\u{1086}\u{1087}\u{108c}\u{108d}\u{108e}\u{109c}\u{109d}\u{109e}\u{135c}\u{135d}\u{135f}\u{1360}\u{1711}\u{1712}\u{1715}\u{1716}\u{1731}\u{1732}\u{1734}\u{1735}\u{1751}\u{1752}\u{1753}\u{1754}\u{1771}\u{1772}\u{1773}\u{1774}\u{17b3}\u{17b4}\u{17b5}\u{17b6}\u{17b7}\u{17bd}\u{17be}\u{17c5}\u{17c6}\u{17c7}\u{17c8}\u{17c9}\u{17d3}\u{17d4}\u{17dc}\u{17dd}\u{17de}\u{180a}\u{180b}\u{180d}\u{180e}\u{180f}\u{1810}\u{1884}\u{1885}\u{1886}\u{1887}\u{18a8}\u{18a9}\u{18aa}\u{191f}\u{1920}\u{1922}\u{1923}\u{1926}\u{1927}\u{1928}\u{1929}\u{1931}\u{1932}\u{1933}\u{1938}\u{1939}\u{193b}\u{193c}\u{1a16}\u{1a17}\u{1a18}\u{1a19}\u{1a1a}\u{1a1b}\u{1a1c}\u{1a55}\u{1a56}\u{1a57}\u{1a58}\u{1a5e}\u{1a5f}\u{1a60}\u{1a61}\u{1a62}\u{1a63}\u{1a64}\u{1a65}\u{1a6c}\u{1a6d}\u{1a72}\u{1a73}\u{1a7c}\u{1a7d}\u{1a7e}\u{1a7f}\u{1a80}\u{1aaf}\u{1ab0}\u{1af1}\u{1aff}\u{1b00}\u{1b03}\u{1b04}\u{1b33}\u{1b34}\u{1b3d}\u{1b3e}\u{1b41}\u{1b42}\u{1b44}\u{1b45}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1b7f}\u{1b80}\u{1b81}\u{1b82}\u{1ba1}\u{1ba2}\u{1ba5}\u{1ba6}\u{1ba7}\u{1ba8}\u{1bad}\u{1bae}\u{1be5}\u{1be6}\u{1be7}\u{1be8}\u{1be9}\u{1bea}\u{1bec}\u{1bed}\u{1bee}\u{1bef}\u{1bf3}\u{1bf4}\u{1c2b}\u{1c2c}\u{1c33}\u{1c34}\u{1c35}\u{1c36}\u{1c37}\u{1c38}\u{1ccf}\u{1cd0}\u{1cd2}\u{1cd3}\u{1cd4}\u{1ce0}\u{1ce1}\u{1ce2}\u{1ce8}\u{1ce9}\u{1cec}\u{1ced}\u{1cee}\u{1cf3}\u{1cf4}\u{1cf5}\u{1cf7}\u{1cf8}\u{1cf9}\u{1cfa}\u{1dbf}\u{1dc0}\u{1dff}\u{1e00}\u{1fbe}\u{200b}\u{200c}\u{200d}\u{20cf}\u{20d0}\u{20f0}\u{20f1}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2d7e}\u{2d7f}\u{2d80}\u{2ddf}\u{2de0}\u{2dff}\u{2e00}\u{3029}\u{302a}\u{302f}\u{3030}\u{3098}\u{3099}\u{309a}\u{309b}\u{a66e}\u{a66f}\u{a672}\u{a673}\u{a674}\u{a67d}\u{a67e}\u{a69d}\u{a69e}\u{a69f}\u{a6a0}\u{a6ef}\u{a6f0}\u{a6f1}\u{a6f2}\u{a801}\u{a802}\u{a803}\u{a805}\u{a806}\u{a807}\u{a80a}\u{a80b}\u{a80c}\u{a824}\u{a825}\u{a826}\u{a827}\u{a82b}\u{a82c}\u{a82d}\u{a8c3}\u{a8c4}\u{a8c5}\u{a8c6}\u{a8df}\u{a8e0}\u{a8f1}\u{a8f2}\u{a8fe}\u{a8ff}\u{a900}\u{a925}\u{a926}\u{a92d}\u{a92e}\u{a946}\u{a947}\u{a951}\u{a952}\u{a953}\u{a954}\u{a97f}\u{a980}\u{a982}\u{a983}\u{a9b2}\u{a9b3}\u{a9b4}\u{a9b5}\u{a9b6}\u{a9b9}\u{a9ba}\u{a9bb}\u{a9bc}\u{a9bd}\u{a9be}\u{a9bf}\u{a9c0}\u{a9c1}\u{a9e4}\u{a9e5}\u{a9e6}\u{aa28}\u{aa29}\u{aa2e}\u{aa2f}\u{aa30}\u{aa31}\u{aa32}\u{aa33}\u{aa34}\u{aa35}\u{aa36}\u{aa37}\u{aa42}\u{aa43}\u{aa44}\u{aa4b}\u{aa4c}\u{aa4d}\u{aa7b}\u{aa7c}\u{aa7d}\u{aaaf}\u{aab0}\u{aab1}\u{aab2}\u{aab4}\u{aab5}\u{aab6}\u{aab7}\u{aab8}\u{aab9}\u{aabd}\u{aabe}\u{aabf}\u{aac0}\u{aac1}\u{aac2}\u{aaeb}\u{aaec}\u{aaed}\u{aaee}\u{aaf5}\u{aaf6}\u{aaf7}\u{abe4}\u{abe5}\u{abe6}\u{abe7}\u{abe8}\u{abe9}\u{abec}\u{abed}\u{abee}\u{fb1d}\u{fb1e}\u{fb1f}\u{fdff}\u{fe00}\u{fe0f}\u{fe10}\u{fe1f}\u{fe20}\u{fe2f}\u{fe30}\u{ff9d}\u{ff9e}\u{ff9f}\u{ffa0}\u{101fc}\u{101fd}\u{101fe}\u{102df}\u{102e0}\u{102e1}\u{10375}\u{10376}\u{1037a}\u{1037b}\u{10a00}\u{10a01}\u{10a03}\u{10a04}\u{10a05}\u{10a06}\u{10a07}\u{10a0b}\u{10a0c}\u{10a0f}\u{10a10}\u{10a37}\u{10a38}\u{10a3a}\u{10a3b}\u{10a3e}\u{10a3f}\u{10a40}\u{10ae4}\u{10ae5}\u{10ae6}\u{10ae7}\u{10d23}\u{10d24}\u{10d27}\u{10d28}\u{10d68}\u{10d69}\u{10d6d}\u{10d6e}\u{10eaa}\u{10eab}\u{10eac}\u{10ead}\u{10eca}\u{10ed0}\u{10eef}\u{10eff}\u{10f00}\u{10f45}\u{10f46}\u{10f50}\u{10f51}\u{10f81}\u{10f82}\u{10f85}\u{10f86}\u{11000}\u{11001}\u{11002}\u{11037}\u{11038}\u{11046}\u{11047}\u{1106f}\u{11070}\u{11071}\u{11072}\u{11073}\u{11074}\u{11075}\u{1107e}\u{1107f}\u{11081}\u{11082}\u{110b2}\u{110b3}\u{110b6}\u{110b7}\u{110b8}\u{110b9}\u{110ba}\u{110bb}\u{110c1}\u{110c2}\u{110c3}\u{110ff}\u{11100}\u{11102}\u{11103}\u{11126}\u{11127}\u{1112b}\u{1112c}\u{1112d}\u{11134}\u{11135}\u{11172}\u{11173}\u{11174}\u{1117f}\u{11180}\u{11181}\u{11182}\u{111b5}\u{111b6}\u{111be}\u{111bf}\u{111c0}\u{111c1}\u{111c8}\u{111c9}\u{111cc}\u{111cd}\u{111ce}\u{111cf}\u{111d0}\u{1122e}\u{1122f}\u{11231}\u{11232}\u{11233}\u{11234}\u{11237}\u{11238}\u{1123d}\u{1123e}\u{1123f}\u{11240}\u{11241}\u{11242}\u{112de}\u{112df}\u{112e0}\u{112e2}\u{112e3}\u{112ea}\u{112eb}\u{112ff}\u{11300}\u{11301}\u{11302}\u{1133a}\u{1133b}\u{1133c}\u{1133d}\u{1133e}\u{1133f}\u{11340}\u{11341}\u{1134c}\u{1134d}\u{1134e}\u{11356}\u{11357}\u{11358}\u{11365}\u{11366}\u{1136c}\u{1136d}\u{1136f}\u{11370}\u{11374}\u{11375}\u{113b7}\u{113b8}\u{113b9}\u{113ba}\u{113bb}\u{113c0}\u{113c1}\u{113c2}\u{113c3}\u{113c4}\u{113c5}\u{113c6}\u{113c7}\u{113c9}\u{113ca}\u{113cd}\u{113ce}\u{113d0}\u{113d1}\u{113d2}\u{113d3}\u{113e0}\u{113e1}\u{113e2}\u{113e3}\u{11437}\u{11438}\u{1143f}\u{11440}\u{11441}\u{11442}\u{11444}\u{11445}\u{11446}\u{11447}\u{1145d}\u{1145e}\u{1145f}\u{114af}\u{114b0}\u{114b1}\u{114b2}\u{114b3}\u{114b8}\u{114b9}\u{114ba}\u{114bb}\u{114bc}\u{114bd}\u{114be}\u{114bf}\u{114c0}\u{114c1}\u{114c2}\u{114c3}\u{114c4}\u{115ae}\u{115af}\u{115b0}\u{115b1}\u{115b2}\u{115b5}\u{115b6}\u{115bb}\u{115bc}\u{115bd}\u{115be}\u{115bf}\u{115c0}\u{115c1}\u{115db}\u{115dc}\u{115dd}\u{115de}\u{11632}\u{11633}\u{1163a}\u{1163b}\u{1163c}\u{1163d}\u{1163e}\u{1163f}\u{11640}\u{11641}\u{116aa}\u{116ab}\u{116ac}\u{116ad}\u{116ae}\u{116af}\u{116b0}\u{116b7}\u{116b8}\u{1171c}\u{1171d}\u{1171e}\u{1171f}\u{11720}\u{11721}\u{11722}\u{11725}\u{11726}\u{11727}\u{1172b}\u{1172c}\u{1182e}\u{1182f}\u{11837}\u{11838}\u{11839}\u{1183a}\u{1183b}\u{1192f}\u{11930}\u{11931}\u{1193a}\u{1193b}\u{1193e}\u{1193f}\u{11942}\u{11943}\u{11944}\u{119d3}\u{119d4}\u{119d7}\u{119d8}\u{119d9}\u{119da}\u{119db}\u{119dc}\u{119df}\u{119e0}\u{119e1}\u{11a00}\u{11a01}\u{11a0a}\u{11a0b}\u{11a32}\u{11a33}\u{11a38}\u{11a39}\u{11a3a}\u{11a3b}\u{11a3e}\u{11a3f}\u{11a46}\u{11a47}\u{11a48}\u{11a50}\u{11a51}\u{11a56}\u{11a57}\u{11a58}\u{11a59}\u{11a5b}\u{11a5c}\u{11a89}\u{11a8a}\u{11a96}\u{11a97}\u{11a98}\u{11a99}\u{11a9a}\u{11b5f}\u{11b60}\u{11b61}\u{11b62}\u{11b64}\u{11b65}\u{11b66}\u{11b67}\u{11c2f}\u{11c30}\u{11c36}\u{11c37}\u{11c38}\u{11c3d}\u{11c3e}\u{11c3f}\u{11c40}\u{11c91}\u{11c92}\u{11ca7}\u{11ca8}\u{11ca9}\u{11caa}\u{11cb0}\u{11cb1}\u{11cb2}\u{11cb3}\u{11cb4}\u{11cb5}\u{11cb6}\u{11cb7}\u{11d30}\u{11d31}\u{11d36}\u{11d37}\u{11d39}\u{11d3a}\u{11d3b}\u{11d3c}\u{11d3d}\u{11d3e}\u{11d3f}\u{11d45}\u{11d46}\u{11d47}\u{11d48}\u{11d8f}\u{11d90}\u{11d91}\u{11d92}\u{11d94}\u{11d95}\u{11d96}\u{11d97}\u{11d98}\u{11def}\u{11df1}\u{11ef2}\u{11ef3}\u{11ef4}\u{11ef5}\u{11eff}\u{11f00}\u{11f01}\u{11f02}\u{11f35}\u{11f36}\u{11f3a}\u{11f3b}\u{11f3f}\u{11f40}\u{11f42}\u{11f43}\u{11f59}\u{11f5a}\u{11f5b}\u{1343f}\u{13440}\u{13441}\u{13446}\u{13447}\u{13455}\u{13456}\u{1611d}\u{1611e}\u{16129}\u{1612a}\u{1612c}\u{1612d}\u{1612f}\u{16130}\u{16aef}\u{16af0}\u{16af4}\u{16af5}\u{16b2f}\u{16b30}\u{16b36}\u{16b37}\u{16f4e}\u{16f4f}\u{16f50}\u{16f8e}\u{16f8f}\u{16f92}\u{16f93}\u{16fe3}\u{16fe4}\u{16fe5}\u{16fef}\u{16ff0}\u{16ff1}\u{16ff2}\u{1bc9c}\u{1bc9d}\u{1bc9e}\u{1bc9f}\u{1ceff}\u{1cf00}\u{1cf2d}\u{1cf2e}\u{1cf2f}\u{1cf30}\u{1cf46}\u{1cf47}\u{1d126}\u{1d129}\u{1d164}\u{1d165}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d241}\u{1d242}\u{1d244}\u{1d245}\u{1d24f}\u{1d253}\u{1d25a}\u{1d25d}\u{1d25e}\u{1d260}\u{1d27f}\u{1d282}\u{1d9ff}\u{1da00}\u{1da36}\u{1da37}\u{1da3a}\u{1da3b}\u{1da6c}\u{1da6d}\u{1da74}\u{1da75}\u{1da76}\u{1da83}\u{1da84}\u{1da85}\u{1da9a}\u{1da9b}\u{1da9f}\u{1daa0}\u{1daa1}\u{1daaf}\u{1dab0}\u{1dfff}\u{1e000}\u{1e006}\u{1e007}\u{1e008}\u{1e018}\u{1e019}\u{1e01a}\u{1e01b}\u{1e021}\u{1e022}\u{1e023}\u{1e024}\u{1e025}\u{1e026}\u{1e02a}\u{1e02b}\u{1e08e}\u{1e08f}\u{1e090}\u{1e12f}\u{1e130}\u{1e136}\u{1e137}\u{1e2ad}\u{1e2ae}\u{1e2af}\u{1e2eb}\u{1e2ec}\u{1e2ef}\u{1e2f0}\u{1e4eb}\u{1e4ec}\u{1e4ef}\u{1e4f0}\u{1e5ed}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e6e2}\u{1e6e3}\u{1e6e4}\u{1e6e5}\u{1e6e6}\u{1e6e7}\u{1e6ed}\u{1e6ee}\u{1e6ef}\u{1e6f0}\u{1e6f4}\u{1e6f5}\u{1e6f6}\u{1e8cf}\u{1e8d0}\u{1e8d6}\u{1e8d7}\u{1e943}\u{1e944}\u{1e94a}\u{1e94b}\u{e001f}\u{e0020}\u{e007f}\u{e0080}\u{e00ff}\u{e0100}\u{e01ef}\u{e01f0}\u{feff}ſK\n",
                "X\r\n\u{300}\u{2028}\u{301}\u{2029}",
                "\u{ff10}\u{ff26}",
                "",
            ] {
                let input = JsString::from(input);
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_grapheme_extend_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Gr_Ext}}$/i{mode},n=/^\P{{Gr_Ext}}$/i{mode};'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Gr_Ext}}K]/i{mode},n=/[\P{{Gr_Ext}}K]/i{mode},x=/[^\P{{Gr_Ext}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Grapheme_Extend}}/{mode},b=/\p{{Gr_Ext}}/i{mode},n=/\P{{Gr_Ext}}/i{mode};a.test('\u0300')&&a.test('\u0301')&&b.test('\u0300')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0300')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Grapheme_Extend}}/{mode},s=/\s/{mode};p.test('\u0300')&&p.test('\u0301')&&!s.test('\u0300')&&!s.test('\u0301')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Gr_Ext}}K]/i{mode},b=/[^\P{{Grapheme_Extend}}K]/i{mode},c=/[\p{{Gr_Ext}}\P{{Grapheme_Extend}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0300')&&!a.test('ſ')&&b.test('\u0301')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Gr_Ext}}$/{mode},b=/^\P{{Gr_Ext}}$/{mode};a.test('\u0300')&&!a.test('\u0301\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Gr_Ext}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD834\uDD65');m.index===0&&m[0]==='\uD834\uDD65'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Gr_Ext}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Gr_Ext}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0300');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Gr_Ext}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0300');m.index===2&&m[0]==='\u0300'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Gr_Ext}}/{mode}y;r.lastIndex=1;r.exec('😀\u0300')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0301😀\u0300G'.matchAll(/(?<x>\P{{Gr_Ext}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u0301\u0300😀X'.replace(/(?<x>\p{{Grapheme_Extend}})/{mode}g,'<$1:$<x>>')==='<\u0301:\u0301><\u0300:\u0300>😀X'"
        ));
        check(&format!(
            r"'😀\u0301\u0300G'.split(/(\p{{Gr_Ext}})/{mode}).join('|')==='😀|\u0301||\u0300|G'&&'😀\u0300'.search(/\p{{Gr_Ext}}/{mode})===2&&'\u0301\u0300😀'.match(/\p{{Gr_Ext}}/{mode}g).join('')==='\u0301\u0300'"
        ));
        check(&format!(
            r"let t='';let x='\u0300\u0301😀\u0300'.replace(/(\p{{Gr_Ext}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Grapheme_Extend})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Grapheme_Extend})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0300').groups.x==='\u0300'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Gr_Ext}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Gr_Ext}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0300')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Gr_Ext}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0300')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_grapheme_extend_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Gr_Ext}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(
        r#"let m=/\p{Script_Extensions=Latin}/u.exec('A');JSON.stringify(m)==='["A"]'&&m.index===0&&m.input==='A'"#,
    );
    check(r"/[\p{Gr_Ext}&&A]/v.exec('A')===null");
    check(r"/[\p{Gr_Ext}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x300),m=/[\p{Gr_Ext}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Gr_Ext}+b/u.exec('AA')", r"/\p{Gr_Ext}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_grapheme_extend_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Gr_Ext}})+\B/d{mode}g,m=r.exec('😀\u0300\u0301\u0301\uFEFF');m.index===2&&m[0]==='\u0300\u0301\u0301'&&m.groups.x==='\u0301'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_grapheme_extend_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x5c8u32, 0x5c9u32, 0xb53u32, 0xb54u32, 0x1adeu32, 0x1adfu32, 0x1aecu32, 0x1aedu32,
        0x1aeeu32, 0x1aefu32, 0x1af0u32, 0x10ecbu32, 0x10eccu32, 0x10ecdu32, 0x10eceu32,
        0x10ecfu32, 0x10ef0u32, 0x10ef1u32, 0x10ef2u32, 0x10ef3u32, 0x10ef4u32, 0x10ef5u32,
        0x10ef6u32, 0x10ef7u32, 0x10ef8u32, 0x10ef9u32, 0x11df0u32, 0x1d127u32, 0x1d128u32,
        0x1d250u32, 0x1d251u32, 0x1d252u32, 0x1d25bu32, 0x1d25cu32, 0x1d25fu32, 0x1d280u32,
        0x1d281u32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Grapheme_Extend", "Gr_Ext"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 296);
}
