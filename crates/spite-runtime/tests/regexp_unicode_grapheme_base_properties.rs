//! Grapheme_Base and its exact Gr_Base alias over complete Unicode characters.
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
fn unicode_grapheme_base_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Grapheme_Base}",
        r"\P{Gr_Base}",
        r"[^\p{Gr_Base}]",
        r"[\p{Gr_Base}K]",
        r"[\P{Grapheme_Base}K]",
        r"^(?<x>[\p{Gr_Base}K])$",
        r"(?<x>\P{Grapheme_Base})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{1f}\u{20}\u{7e}\u{7f}\u{9f}\u{a0}\u{ac}\u{ad}\u{ae}\u{2ff}\u{300}\u{345}\u{36f}\u{370}\u{377}\u{378}\u{379}\u{37a}\u{37f}\u{380}\u{383}\u{384}\u{38a}\u{38b}\u{38c}\u{38d}\u{38e}\u{399}\u{3a1}\u{3a2}\u{3a3}\u{3b9}\u{482}\u{483}\u{489}\u{48a}\u{52f}\u{530}\u{531}\u{556}\u{557}\u{58f}\u{590}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5cf}\u{5d0}\u{5ea}\u{5eb}\u{5ee}\u{5ef}\u{5f4}\u{5f5}\u{605}\u{606}\u{60f}\u{610}\u{61a}\u{61b}\u{61c}\u{61d}\u{64a}\u{64b}\u{65f}\u{660}\u{66f}\u{670}\u{671}\u{6d5}\u{6d6}\u{6dd}\u{6de}\u{6df}\u{6e4}\u{6e5}\u{6e6}\u{6e7}\u{6e8}\u{6e9}\u{6ea}\u{6ed}\u{6ee}\u{70d}\u{70e}\u{70f}\u{710}\u{711}\u{712}\u{72f}\u{730}\u{74c}\u{74d}\u{7a5}\u{7a6}\u{7b0}\u{7b1}\u{7b2}\u{7bf}\u{7c0}\u{7ea}\u{7eb}\u{7f3}\u{7f4}\u{7fa}\u{7fb}\u{7fd}\u{7fe}\u{815}\u{816}\u{819}\u{81a}\u{81b}\u{823}\u{824}\u{825}\u{827}\u{828}\u{829}\u{82f}\u{830}\u{83e}\u{83f}\u{840}\u{858}\u{859}\u{85d}\u{85e}\u{85f}\u{860}\u{86a}\u{86b}\u{86f}\u{870}\u{88f}\u{890}\u{89f}\u{8a0}\u{8c9}\u{8ca}\u{902}\u{903}\u{939}\u{93a}\u{93b}\u{93c}\u{93d}\u{940}\u{941}\u{948}\u{949}\u{94c}\u{94d}\u{94e}\u{950}\u{951}\u{957}\u{958}\u{961}\u{962}\u{963}\u{964}\u{980}\u{981}\u{982}\u{983}\u{984}\u{985}\u{98c}\u{98d}\u{98e}\u{98f}\u{990}\u{991}\u{992}\u{993}\u{9a8}\u{9a9}\u{9aa}\u{9b0}\u{9b1}\u{9b2}\u{9b3}\u{9b5}\u{9b6}\u{9b9}\u{9ba}\u{9bc}\u{9bd}\u{9be}\u{9bf}\u{9c0}\u{9c1}\u{9c6}\u{9c7}\u{9c8}\u{9c9}\u{9ca}\u{9cb}\u{9cc}\u{9cd}\u{9ce}\u{9cf}\u{9db}\u{9dc}\u{9dd}\u{9de}\u{9df}\u{9e1}\u{9e2}\u{9e5}\u{9e6}\u{9fd}\u{9fe}\u{a02}\u{a03}\u{a04}\u{a05}\u{a0a}\u{a0b}\u{a0e}\u{a0f}\u{a10}\u{a11}\u{a12}\u{a13}\u{a28}\u{a29}\u{a2a}\u{a30}\u{a31}\u{a32}\u{a33}\u{a34}\u{a35}\u{a36}\u{a37}\u{a38}\u{a39}\u{a3a}\u{a3d}\u{a3e}\u{a40}\u{a41}\u{a58}\u{a59}\u{a5c}\u{a5d}\u{a5e}\u{a5f}\u{a65}\u{a66}\u{a6f}\u{a70}\u{a71}\u{a72}\u{a74}\u{a75}\u{a76}\u{a77}\u{a82}\u{a83}\u{a84}\u{a85}\u{a8d}\u{a8e}\u{a8f}\u{a91}\u{a92}\u{a93}\u{aa8}\u{aa9}\u{aaa}\u{ab0}\u{ab1}\u{ab2}\u{ab3}\u{ab4}\u{ab5}\u{ab9}\u{aba}\u{abc}\u{abd}\u{ac0}\u{ac1}\u{ac8}\u{ac9}\u{aca}\u{acb}\u{acc}\u{acd}\u{acf}\u{ad0}\u{ad1}\u{adf}\u{ae0}\u{ae1}\u{ae2}\u{ae5}\u{ae6}\u{af1}\u{af2}\u{af8}\u{af9}\u{afa}\u{b01}\u{b02}\u{b03}\u{b04}\u{b05}\u{b0c}\u{b0d}\u{b0e}\u{b0f}\u{b10}\u{b11}\u{b12}\u{b13}\u{b28}\u{b29}\u{b2a}\u{b30}\u{b31}\u{b32}\u{b33}\u{b34}\u{b35}\u{b39}\u{b3a}\u{b3c}\u{b3d}\u{b3e}\u{b3f}\u{b40}\u{b41}\u{b46}\u{b47}\u{b48}\u{b49}\u{b4a}\u{b4b}\u{b4c}\u{b4d}\u{b5b}\u{b5c}\u{b5d}\u{b5e}\u{b5f}\u{b61}\u{b62}\u{b65}\u{b66}\u{b77}\u{b78}\u{b82}\u{b83}\u{b84}\u{b85}\u{b8a}\u{b8b}\u{b8d}\u{b8e}\u{b90}\u{b91}\u{b92}\u{b95}\u{b96}\u{b98}\u{b99}\u{b9a}\u{b9b}\u{b9c}\u{b9d}\u{b9e}\u{b9f}\u{ba0}\u{ba2}\u{ba3}\u{ba4}\u{ba5}\u{ba7}\u{ba8}\u{baa}\u{bab}\u{bad}\u{bae}\u{bb9}\u{bba}\u{bbe}\u{bbf}\u{bc0}\u{bc1}\u{bc2}\u{bc3}\u{bc5}\u{bc6}\u{bc8}\u{bc9}\u{bca}\u{bcc}\u{bcd}\u{bcf}\u{bd0}\u{bd1}\u{be5}\u{be6}\u{bfa}\u{bfb}\u{c00}\u{c01}\u{c03}\u{c04}\u{c05}\u{c0c}\u{c0d}\u{c0e}\u{c10}\u{c11}\u{c12}\u{c28}\u{c29}\u{c2a}\u{c39}\u{c3a}\u{c3c}\u{c3d}\u{c3e}\u{c40}\u{c41}\u{c44}\u{c45}\u{c57}\u{c58}\u{c5a}\u{c5b}\u{c5c}\u{c5d}\u{c5e}\u{c5f}\u{c60}\u{c61}\u{c62}\u{c65}\u{c66}\u{c6f}\u{c70}\u{c76}\u{c77}\u{c80}\u{c81}\u{c82}\u{c8c}\u{c8d}\u{c8e}\u{c90}\u{c91}\u{c92}\u{ca8}\u{ca9}\u{caa}\u{cb3}\u{cb4}\u{cb5}\u{cb9}\u{cba}\u{cbc}\u{cbd}\u{cbe}\u{cbf}\u{cc0}\u{cc1}\u{cc2}\u{cc3}\u{cc4}\u{cc5}\u{cdb}\u{cdc}\u{cde}\u{cdf}\u{ce0}\u{ce1}\u{ce2}\u{ce5}\u{ce6}\u{cef}\u{cf0}\u{cf1}\u{cf3}\u{cf4}\u{d01}\u{d02}\u{d0c}\u{d0d}\u{d0e}\u{d10}\u{d11}\u{d12}\u{d3a}\u{d3b}\u{d3c}\u{d3d}\u{d3e}\u{d3f}\u{d40}\u{d41}\u{d45}\u{d46}\u{d48}\u{d49}\u{d4a}\u{d4c}\u{d4d}\u{d4e}\u{d4f}\u{d50}\u{d53}\u{d54}\u{d56}\u{d57}\u{d58}\u{d61}\u{d62}\u{d65}\u{d66}\u{d7f}\u{d80}\u{d81}\u{d82}\u{d83}\u{d84}\u{d85}\u{d96}\u{d97}\u{d99}\u{d9a}\u{db1}\u{db2}\u{db3}\u{dbb}\u{dbc}\u{dbd}\u{dbe}\u{dbf}\u{dc0}\u{dc6}\u{dc7}\u{dcf}\u{dd0}\u{dd1}\u{dd2}\u{dd7}\u{dd8}\u{dde}\u{ddf}\u{de5}\u{de6}\u{def}\u{df0}\u{df1}\u{df2}\u{df4}\u{df5}\u{e00}\u{e01}\u{e30}\u{e31}\u{e32}\u{e33}\u{e34}\u{e3e}\u{e3f}\u{e46}\u{e47}\u{e4e}\u{e4f}\u{e5b}\u{e5c}\u{e80}\u{e81}\u{e82}\u{e83}\u{e84}\u{e85}\u{e86}\u{e8a}\u{e8b}\u{e8c}\u{ea3}\u{ea4}\u{ea5}\u{ea6}\u{ea7}\u{eb0}\u{eb1}\u{eb2}\u{eb3}\u{eb4}\u{ebc}\u{ebd}\u{ebe}\u{ebf}\u{ec0}\u{ec4}\u{ec5}\u{ec6}\u{ec7}\u{ecf}\u{ed0}\u{ed9}\u{eda}\u{edb}\u{edc}\u{edf}\u{ee0}\u{eff}\u{f00}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f47}\u{f48}\u{f49}\u{f6c}\u{f6d}\u{f7e}\u{f7f}\u{f80}\u{f84}\u{f85}\u{f86}\u{f87}\u{f88}\u{f8c}\u{f8d}\u{fbd}\u{fbe}\u{fc5}\u{fc6}\u{fc7}\u{fcc}\u{fcd}\u{fce}\u{fda}\u{fdb}\u{fff}\u{1000}\u{102c}\u{102d}\u{1030}\u{1031}\u{1032}\u{1037}\u{1038}\u{1039}\u{103a}\u{103b}\u{103c}\u{103d}\u{103e}\u{103f}\u{1057}\u{1058}\u{1059}\u{105a}\u{105d}\u{105e}\u{1060}\u{1061}\u{1070}\u{1071}\u{1074}\u{1075}\u{1081}\u{1082}\u{1083}\u{1084}\u{1085}\u{1086}\u{1087}\u{108c}\u{108d}\u{108e}\u{109c}\u{109d}\u{109e}\u{10c5}\u{10c6}\u{10c7}\u{10c8}\u{10cc}\u{10cd}\u{10ce}\u{10cf}\u{10d0}\u{1248}\u{1249}\u{124a}\u{124d}\u{124e}\u{124f}\u{1250}\u{1256}\u{1257}\u{1258}\u{1259}\u{125a}\u{125d}\u{125e}\u{125f}\u{1260}\u{1288}\u{1289}\u{128a}\u{128d}\u{128e}\u{128f}\u{1290}\u{12b0}\u{12b1}\u{12b2}\u{12b5}\u{12b6}\u{12b7}\u{12b8}\u{12be}\u{12bf}\u{12c0}\u{12c1}\u{12c2}\u{12c5}\u{12c6}\u{12c7}\u{12c8}\u{12d6}\u{12d7}\u{12d8}\u{1310}\u{1311}\u{1312}\u{1315}\u{1316}\u{1317}\u{1318}\u{135a}\u{135b}\u{135f}\u{1360}\u{137c}\u{137d}\u{137f}\u{1380}\u{1399}\u{139a}\u{139f}\u{13a0}\u{13f5}\u{13f6}\u{13f7}\u{13f8}\u{13fd}\u{13fe}\u{13ff}\u{1400}\u{169c}\u{169d}\u{169f}\u{16a0}\u{16f8}\u{16f9}\u{16ff}\u{1700}\u{1711}\u{1712}\u{171e}\u{171f}\u{1731}\u{1732}\u{1734}\u{1735}\u{1736}\u{1737}\u{173f}\u{1740}\u{1751}\u{1752}\u{175f}\u{1760}\u{176c}\u{176d}\u{176e}\u{1770}\u{1771}\u{177f}\u{1780}\u{17b3}\u{17b4}\u{17b5}\u{17b6}\u{17b7}\u{17bd}\u{17be}\u{17c5}\u{17c6}\u{17c7}\u{17c8}\u{17c9}\u{17d3}\u{17d4}\u{17dc}\u{17dd}\u{17df}\u{17e0}\u{17e9}\u{17ea}\u{17ef}\u{17f0}\u{17f9}\u{17fa}\u{17ff}\u{1800}\u{180a}\u{180b}\u{180f}\u{1810}\u{1819}\u{181a}\u{181f}\u{1820}\u{1878}\u{1879}\u{187f}\u{1880}\u{1884}\u{1885}\u{1886}\u{1887}\u{18a8}\u{18a9}\u{18aa}\u{18ab}\u{18af}\u{18b0}\u{18f5}\u{18f6}\u{18ff}\u{1900}\u{191e}\u{191f}\u{1922}\u{1923}\u{1926}\u{1927}\u{1928}\u{1929}\u{192b}\u{192c}\u{192f}\u{1930}\u{1931}\u{1932}\u{1933}\u{1938}\u{1939}\u{193f}\u{1940}\u{1941}\u{1943}\u{1944}\u{196d}\u{196e}\u{196f}\u{1970}\u{1974}\u{1975}\u{197f}\u{1980}\u{19ab}\u{19ac}\u{19af}\u{19b0}\u{19c9}\u{19ca}\u{19cf}\u{19d0}\u{19da}\u{19db}\u{19dd}\u{19de}\u{1a16}\u{1a17}\u{1a18}\u{1a19}\u{1a1a}\u{1a1b}\u{1a1d}\u{1a1e}\u{1a55}\u{1a56}\u{1a57}\u{1a58}\u{1a60}\u{1a61}\u{1a62}\u{1a63}\u{1a64}\u{1a65}\u{1a6c}\u{1a6d}\u{1a72}\u{1a73}\u{1a7f}\u{1a80}\u{1a89}\u{1a8a}\u{1a8f}\u{1a90}\u{1a99}\u{1a9a}\u{1a9f}\u{1aa0}\u{1aad}\u{1aae}\u{1b03}\u{1b04}\u{1b33}\u{1b34}\u{1b3d}\u{1b3e}\u{1b41}\u{1b42}\u{1b44}\u{1b45}\u{1b4c}\u{1b4d}\u{1b4e}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1b7f}\u{1b80}\u{1b81}\u{1b82}\u{1ba1}\u{1ba2}\u{1ba5}\u{1ba6}\u{1ba7}\u{1ba8}\u{1bad}\u{1bae}\u{1be5}\u{1be6}\u{1be7}\u{1be8}\u{1be9}\u{1bea}\u{1bec}\u{1bed}\u{1bee}\u{1bef}\u{1bfb}\u{1bfc}\u{1c2b}\u{1c2c}\u{1c33}\u{1c34}\u{1c35}\u{1c36}\u{1c3a}\u{1c3b}\u{1c49}\u{1c4a}\u{1c4c}\u{1c4d}\u{1c8a}\u{1c8b}\u{1c8f}\u{1c90}\u{1cba}\u{1cbb}\u{1cbc}\u{1cbd}\u{1cc7}\u{1cc8}\u{1cd2}\u{1cd3}\u{1cd4}\u{1ce0}\u{1ce1}\u{1ce2}\u{1ce8}\u{1ce9}\u{1cec}\u{1ced}\u{1cee}\u{1cf3}\u{1cf4}\u{1cf5}\u{1cf7}\u{1cf8}\u{1cf9}\u{1cfa}\u{1cfb}\u{1cff}\u{1d00}\u{1dbf}\u{1dc0}\u{1dff}\u{1e00}\u{1f15}\u{1f16}\u{1f17}\u{1f18}\u{1f1d}\u{1f1e}\u{1f1f}\u{1f20}\u{1f45}\u{1f46}\u{1f47}\u{1f48}\u{1f4d}\u{1f4e}\u{1f4f}\u{1f50}\u{1f57}\u{1f58}\u{1f59}\u{1f5a}\u{1f5b}\u{1f5c}\u{1f5d}\u{1f5e}\u{1f5f}\u{1f7d}\u{1f7e}\u{1f7f}\u{1f80}\u{1fb4}\u{1fb5}\u{1fb6}\u{1fbe}\u{1fc4}\u{1fc5}\u{1fc6}\u{1fd3}\u{1fd4}\u{1fd5}\u{1fd6}\u{1fdb}\u{1fdc}\u{1fdd}\u{1fef}\u{1ff0}\u{1ff1}\u{1ff2}\u{1ff4}\u{1ff5}\u{1ff6}\u{1ffe}\u{1fff}\u{2000}\u{200a}\u{200b}\u{200f}\u{2010}\u{2027}\u{2028}\u{202e}\u{202f}\u{205f}\u{2060}\u{206f}\u{2070}\u{2071}\u{2072}\u{2073}\u{2074}\u{20c5}\u{20ff}\u{2100}\u{218b}\u{218c}\u{218f}\u{2190}\u{2429}\u{242a}\u{243f}\u{2440}\u{244a}\u{244b}\u{245f}\u{2460}\u{2b73}\u{2b74}\u{2b75}\u{2b76}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2cf3}\u{2cf4}\u{2cf8}\u{2cf9}\u{2d25}\u{2d26}\u{2d27}\u{2d28}\u{2d2c}\u{2d2d}\u{2d2e}\u{2d2f}\u{2d30}\u{2d67}\u{2d68}\u{2d6e}\u{2d6f}\u{2d70}\u{2d71}\u{2d7f}\u{2d80}\u{2d96}\u{2d97}\u{2d9f}\u{2da0}\u{2da6}\u{2da7}\u{2da8}\u{2dae}\u{2daf}\u{2db0}\u{2db6}\u{2db7}\u{2db8}\u{2dbe}\u{2dbf}\u{2dc0}\u{2dc6}\u{2dc7}\u{2dc8}\u{2dce}\u{2dcf}\u{2dd0}\u{2dd6}\u{2dd7}\u{2dd8}\u{2dde}\u{2ddf}\u{2dff}\u{2e00}\u{2e5d}\u{2e5e}\u{2e5f}\u{2e64}\u{2e7f}\u{2e80}\u{2e99}\u{2e9a}\u{2e9b}\u{2ef3}\u{2ef4}\u{2eff}\u{2f00}\u{2fd5}\u{2fd6}\u{2fef}\u{2ff0}\u{3029}\u{302a}\u{302f}\u{3030}\u{303f}\u{3040}\u{3041}\u{3096}\u{3097}\u{309a}\u{309b}\u{30ff}\u{3100}\u{3104}\u{3105}\u{312f}\u{3130}\u{3131}\u{318e}\u{318f}\u{3190}\u{31e5}\u{31e6}\u{31ee}\u{31ef}\u{321e}\u{321f}\u{3220}\u{a48c}\u{a48d}\u{a48f}\u{a490}\u{a4c6}\u{a4c7}\u{a4cf}\u{a4d0}\u{a62b}\u{a62c}\u{a63f}\u{a640}\u{a66e}\u{a66f}\u{a672}\u{a673}\u{a674}\u{a67d}\u{a67e}\u{a69d}\u{a69e}\u{a69f}\u{a6a0}\u{a6ef}\u{a6f0}\u{a6f1}\u{a6f2}\u{a6f7}\u{a6f8}\u{a6ff}\u{a700}\u{a7de}\u{a7e1}\u{a7e3}\u{a7f0}\u{a7f1}\u{a801}\u{a802}\u{a803}\u{a805}\u{a806}\u{a807}\u{a80a}\u{a80b}\u{a80c}\u{a824}\u{a825}\u{a826}\u{a827}\u{a82b}\u{a82c}\u{a82f}\u{a830}\u{a839}\u{a83a}\u{a83f}\u{a840}\u{a877}\u{a878}\u{a87f}\u{a880}\u{a8c3}\u{a8c4}\u{a8cd}\u{a8ce}\u{a8d9}\u{a8da}\u{a8f1}\u{a8f2}\u{a8fe}\u{a8ff}\u{a900}\u{a925}\u{a926}\u{a92d}\u{a92e}\u{a946}\u{a947}\u{a951}\u{a952}\u{a953}\u{a95e}\u{a95f}\u{a97c}\u{a97d}\u{a982}\u{a983}\u{a9b2}\u{a9b3}\u{a9b4}\u{a9b5}\u{a9b6}\u{a9b9}\u{a9ba}\u{a9bb}\u{a9bc}\u{a9bd}\u{a9be}\u{a9bf}\u{a9c0}\u{a9c1}\u{a9cd}\u{a9ce}\u{a9cf}\u{a9d9}\u{a9da}\u{a9dd}\u{a9de}\u{a9e4}\u{a9e5}\u{a9e6}\u{a9fe}\u{a9ff}\u{aa00}\u{aa28}\u{aa29}\u{aa2e}\u{aa2f}\u{aa30}\u{aa31}\u{aa32}\u{aa33}\u{aa34}\u{aa35}\u{aa3f}\u{aa40}\u{aa42}\u{aa43}\u{aa44}\u{aa4b}\u{aa4c}\u{aa4d}\u{aa4e}\u{aa4f}\u{aa50}\u{aa59}\u{aa5a}\u{aa5b}\u{aa5c}\u{aa7b}\u{aa7c}\u{aa7d}\u{aaaf}\u{aab0}\u{aab1}\u{aab2}\u{aab4}\u{aab5}\u{aab6}\u{aab7}\u{aab8}\u{aab9}\u{aabd}\u{aabe}\u{aabf}\u{aac0}\u{aac1}\u{aac2}\u{aac3}\u{aada}\u{aadb}\u{aaeb}\u{aaec}\u{aaed}\u{aaee}\u{aaf5}\u{aaf6}\u{ab00}\u{ab01}\u{ab06}\u{ab07}\u{ab08}\u{ab09}\u{ab0e}\u{ab0f}\u{ab10}\u{ab11}\u{ab16}\u{ab17}\u{ab1f}\u{ab20}\u{ab26}\u{ab27}\u{ab28}\u{ab2e}\u{ab2f}\u{ab30}\u{ab6e}\u{ab6f}\u{ab70}\u{abe4}\u{abe5}\u{abe6}\u{abe7}\u{abe8}\u{abe9}\u{abec}\u{abed}\u{abef}\u{abf0}\u{abf9}\u{abfa}\u{abff}\u{ac00}\u{d7a3}\u{d7a4}\u{d7af}\u{d7b0}\u{d7c6}\u{d7c7}\u{d7ca}\u{d7cb}\u{d7fb}\u{d7fc}\u{f8ff}\u{f900}\u{fa6d}\u{fa6e}\u{fa6f}\u{fa70}\u{fad9}\u{fada}\u{faff}\u{fb00}\u{fb06}\u{fb07}\u{fb12}\u{fb13}\u{fb17}\u{fb18}\u{fb1c}\u{fb1d}\u{fb1e}\u{fb1f}\u{fb36}\u{fb37}\u{fb38}\u{fb3c}\u{fb3d}\u{fb3e}\u{fb3f}\u{fb40}\u{fb41}\u{fb42}\u{fb43}\u{fb44}\u{fb45}\u{fb46}\u{fdcf}\u{fdd0}\u{fdef}\u{fdf0}\u{fdff}\u{fe00}\u{fe0f}\u{fe10}\u{fe19}\u{fe1a}\u{fe2f}\u{fe30}\u{fe52}\u{fe53}\u{fe54}\u{fe66}\u{fe67}\u{fe68}\u{fe6b}\u{fe6c}\u{fe6f}\u{fe70}\u{fe74}\u{fe75}\u{fe76}\u{fefc}\u{fefd}\u{ff00}\u{ff01}\u{ff9d}\u{ff9e}\u{ff9f}\u{ffa0}\u{ffbe}\u{ffbf}\u{ffc1}\u{ffc2}\u{ffc7}\u{ffc8}\u{ffc9}\u{ffca}\u{ffcf}\u{ffd0}\u{ffd1}\u{ffd2}\u{ffd7}\u{ffd8}\u{ffd9}\u{ffda}\u{ffdc}\u{ffdd}\u{ffdf}\u{ffe0}\u{ffe6}\u{ffe7}\u{ffe8}\u{ffee}\u{ffef}\u{fffb}\u{fffc}\u{fffd}\u{fffe}\u{ffff}\u{10000}\u{1000b}\u{1000c}\u{1000d}\u{10026}\u{10027}\u{10028}\u{1003a}\u{1003b}\u{1003c}\u{1003d}\u{1003e}\u{1003f}\u{1004d}\u{1004e}\u{1004f}\u{10050}\u{1005d}\u{1005e}\u{1007f}\u{10080}\u{100fa}\u{100fb}\u{100ff}\u{10100}\u{10102}\u{10103}\u{10106}\u{10107}\u{10133}\u{10134}\u{10136}\u{10137}\u{1018e}\u{1018f}\u{10190}\u{1019c}\u{1019d}\u{1019f}\u{101a0}\u{101a1}\u{101cf}\u{101d0}\u{101fc}\u{101fd}\u{1027f}\u{10280}\u{1029c}\u{1029d}\u{1029f}\u{102a0}\u{102d0}\u{102d1}\u{102e0}\u{102e1}\u{102fb}\u{102fc}\u{102ff}\u{10300}\u{10323}\u{10324}\u{1032c}\u{1032d}\u{1034a}\u{1034b}\u{1034f}\u{10350}\u{10375}\u{10376}\u{1037f}\u{10380}\u{1039d}\u{1039e}\u{1039f}\u{103c3}\u{103c4}\u{103c7}\u{103c8}\u{103d5}\u{103d6}\u{103ff}\u{10400}\u{1049d}\u{1049e}\u{1049f}\u{104a0}\u{104a9}\u{104aa}\u{104af}\u{104b0}\u{104d3}\u{104d4}\u{104d7}\u{104d8}\u{104fb}\u{104fc}\u{104ff}\u{10500}\u{10527}\u{10528}\u{1052f}\u{10530}\u{10563}\u{10564}\u{1056e}\u{1056f}\u{1057a}\u{1057b}\u{1057c}\u{1058a}\u{1058b}\u{1058c}\u{10592}\u{10593}\u{10594}\u{10595}\u{10596}\u{10597}\u{105a1}\u{105a2}\u{105a3}\u{105b1}\u{105b2}\u{105b3}\u{105b9}\u{105ba}\u{105bb}\u{105bc}\u{105bd}\u{105bf}\u{105c0}\u{105f3}\u{105f4}\u{105ff}\u{10600}\u{10736}\u{10737}\u{1073f}\u{10740}\u{10755}\u{10756}\u{1075f}\u{10760}\u{10767}\u{10768}\u{1077f}\u{10780}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107c0}\u{107ff}\u{10800}\u{10805}\u{10806}\u{10807}\u{10808}\u{10809}\u{1080a}\u{10835}\u{10836}\u{10837}\u{10838}\u{10839}\u{1083b}\u{1083c}\u{1083d}\u{1083e}\u{1083f}\u{10855}\u{10856}\u{10857}\u{1089e}\u{1089f}\u{108a6}\u{108a7}\u{108af}\u{108b0}\u{108df}\u{108e0}\u{108f2}\u{108f3}\u{108f4}\u{108f5}\u{108f6}\u{108fa}\u{108fb}\u{1091b}\u{1091c}\u{1091e}\u{1091f}\u{10939}\u{1093a}\u{1093e}\u{1093f}\u{10959}\u{1095a}\u{1097f}\u{10980}\u{109b7}\u{109b8}\u{109bb}\u{109bc}\u{109cf}\u{109d0}\u{109d1}\u{109d2}\u{10a00}\u{10a01}\u{10a0f}\u{10a10}\u{10a13}\u{10a14}\u{10a15}\u{10a17}\u{10a18}\u{10a19}\u{10a35}\u{10a36}\u{10a3f}\u{10a40}\u{10a48}\u{10a49}\u{10a4f}\u{10a50}\u{10a58}\u{10a59}\u{10a5f}\u{10a60}\u{10a9f}\u{10aa0}\u{10abf}\u{10ac0}\u{10ae4}\u{10ae5}\u{10aea}\u{10aeb}\u{10af6}\u{10af7}\u{10aff}\u{10b00}\u{10b35}\u{10b36}\u{10b38}\u{10b39}\u{10b55}\u{10b56}\u{10b57}\u{10b58}\u{10b72}\u{10b73}\u{10b77}\u{10b78}\u{10b91}\u{10b92}\u{10b98}\u{10b99}\u{10b9c}\u{10b9d}\u{10ba8}\u{10ba9}\u{10baf}\u{10bb0}\u{10bff}\u{10c00}\u{10c48}\u{10c49}\u{10c7f}\u{10c80}\u{10cb2}\u{10cb3}\u{10cbf}\u{10cc0}\u{10cf2}\u{10cf3}\u{10cf9}\u{10cfa}\u{10d23}\u{10d24}\u{10d2f}\u{10d30}\u{10d39}\u{10d3a}\u{10d3f}\u{10d40}\u{10d65}\u{10d66}\u{10d6d}\u{10d6e}\u{10d85}\u{10d86}\u{10d8d}\u{10d8e}\u{10d8f}\u{10d90}\u{10e5f}\u{10e60}\u{10e7e}\u{10e7f}\u{10e80}\u{10ea9}\u{10eaa}\u{10eac}\u{10ead}\u{10eae}\u{10eaf}\u{10eb0}\u{10eb1}\u{10eb2}\u{10ec1}\u{10ec2}\u{10ec7}\u{10ec8}\u{10ecb}\u{10ecf}\u{10ed0}\u{10eef}\u{10eff}\u{10f00}\u{10f27}\u{10f28}\u{10f2f}\u{10f30}\u{10f45}\u{10f46}\u{10f50}\u{10f51}\u{10f59}\u{10f5a}\u{10f6f}\u{10f70}\u{10f81}\u{10f82}\u{10f85}\u{10f86}\u{10f89}\u{10f8a}\u{10faf}\u{10fb0}\u{10fcb}\u{10fcc}\u{10fdf}\u{10fe0}\u{10ff6}\u{10ff7}\u{10fff}\u{11000}\u{11001}\u{11002}\u{11037}\u{11038}\u{11046}\u{11047}\u{1104d}\u{1104e}\u{11051}\u{11052}\u{1106f}\u{11070}\u{11071}\u{11072}\u{11073}\u{11074}\u{11075}\u{11076}\u{11081}\u{11082}\u{110b2}\u{110b3}\u{110b6}\u{110b7}\u{110b8}\u{110b9}\u{110ba}\u{110bb}\u{110bc}\u{110bd}\u{110be}\u{110c1}\u{110c2}\u{110cf}\u{110d0}\u{110e8}\u{110e9}\u{110ef}\u{110f0}\u{110f9}\u{110fa}\u{11102}\u{11103}\u{11126}\u{11127}\u{1112b}\u{1112c}\u{1112d}\u{11135}\u{11136}\u{11147}\u{11148}\u{1114f}\u{11150}\u{11172}\u{11173}\u{11174}\u{11176}\u{11177}\u{11181}\u{11182}\u{111b5}\u{111b6}\u{111be}\u{111bf}\u{111c0}\u{111c1}\u{111c8}\u{111c9}\u{111cc}\u{111cd}\u{111ce}\u{111cf}\u{111d0}\u{111df}\u{111e0}\u{111e1}\u{111f4}\u{111f5}\u{111ff}\u{11200}\u{11211}\u{11212}\u{11213}\u{1122e}\u{1122f}\u{11231}\u{11232}\u{11233}\u{11234}\u{11237}\u{11238}\u{1123d}\u{1123e}\u{1123f}\u{11240}\u{11241}\u{1127f}\u{11280}\u{11286}\u{11287}\u{11288}\u{11289}\u{1128a}\u{1128d}\u{1128e}\u{1128f}\u{1129d}\u{1129e}\u{1129f}\u{112a9}\u{112aa}\u{112af}\u{112b0}\u{112de}\u{112df}\u{112e0}\u{112e2}\u{112e3}\u{112ef}\u{112f0}\u{112f9}\u{112fa}\u{11301}\u{11302}\u{11303}\u{11304}\u{11305}\u{1130c}\u{1130d}\u{1130e}\u{1130f}\u{11310}\u{11311}\u{11312}\u{11313}\u{11328}\u{11329}\u{1132a}\u{11330}\u{11331}\u{11332}\u{11333}\u{11334}\u{11335}\u{11339}\u{1133a}\u{1133c}\u{1133d}\u{1133e}\u{1133f}\u{11340}\u{11341}\u{11344}\u{11345}\u{11346}\u{11347}\u{11348}\u{11349}\u{1134a}\u{1134b}\u{1134c}\u{1134d}\u{1134f}\u{11350}\u{11351}\u{1135c}\u{1135d}\u{11363}\u{11364}\u{1137f}\u{11380}\u{11389}\u{1138a}\u{1138b}\u{1138c}\u{1138d}\u{1138e}\u{1138f}\u{11390}\u{113b5}\u{113b6}\u{113b7}\u{113b8}\u{113b9}\u{113ba}\u{113bb}\u{113c9}\u{113ca}\u{113cb}\u{113cc}\u{113cd}\u{113ce}\u{113d0}\u{113d1}\u{113d2}\u{113d3}\u{113d5}\u{113d6}\u{113d7}\u{113d8}\u{113d9}\u{113ff}\u{11400}\u{11437}\u{11438}\u{1143f}\u{11440}\u{11441}\u{11442}\u{11444}\u{11445}\u{11446}\u{11447}\u{1145b}\u{1145c}\u{1145d}\u{1145e}\u{1145f}\u{11461}\u{11462}\u{1147f}\u{11480}\u{114af}\u{114b0}\u{114b1}\u{114b2}\u{114b3}\u{114b8}\u{114b9}\u{114ba}\u{114bb}\u{114bc}\u{114bd}\u{114be}\u{114bf}\u{114c0}\u{114c1}\u{114c2}\u{114c3}\u{114c4}\u{114c7}\u{114c8}\u{114cf}\u{114d0}\u{114d9}\u{114da}\u{1157f}\u{11580}\u{115ae}\u{115af}\u{115b0}\u{115b1}\u{115b2}\u{115b7}\u{115b8}\u{115bb}\u{115bc}\u{115bd}\u{115be}\u{115bf}\u{115c0}\u{115c1}\u{115db}\u{115dc}\u{115ff}\u{11600}\u{11632}\u{11633}\u{1163a}\u{1163b}\u{1163c}\u{1163d}\u{1163e}\u{1163f}\u{11640}\u{11641}\u{11644}\u{11645}\u{1164f}\u{11650}\u{11659}\u{1165a}\u{1165f}\u{11660}\u{1166c}\u{1166d}\u{1167f}\u{11680}\u{116aa}\u{116ab}\u{116ac}\u{116ad}\u{116ae}\u{116af}\u{116b0}\u{116b7}\u{116b8}\u{116b9}\u{116ba}\u{116bf}\u{116c0}\u{116c9}\u{116ca}\u{116cf}\u{116d0}\u{116e3}\u{116e4}\u{116ff}\u{11700}\u{1171a}\u{1171b}\u{1171d}\u{1171e}\u{1171f}\u{11720}\u{11721}\u{11722}\u{11725}\u{11726}\u{11727}\u{1172f}\u{11730}\u{11746}\u{11747}\u{117ff}\u{11800}\u{1182e}\u{1182f}\u{11837}\u{11838}\u{11839}\u{1183a}\u{1183b}\u{1183c}\u{1189f}\u{118a0}\u{118f2}\u{118f3}\u{118fe}\u{118ff}\u{11906}\u{11907}\u{11908}\u{11909}\u{1190a}\u{1190b}\u{1190c}\u{11913}\u{11914}\u{11915}\u{11916}\u{11917}\u{11918}\u{1192f}\u{11930}\u{11931}\u{11935}\u{11936}\u{11937}\u{11938}\u{11939}\u{1193e}\u{1193f}\u{11942}\u{11943}\u{11944}\u{11946}\u{11947}\u{1194f}\u{11950}\u{11959}\u{1195a}\u{1199f}\u{119a0}\u{119a7}\u{119a8}\u{119a9}\u{119aa}\u{119d3}\u{119d4}\u{119db}\u{119dc}\u{119df}\u{119e0}\u{119e1}\u{119e4}\u{119e5}\u{119ff}\u{11a00}\u{11a01}\u{11a0a}\u{11a0b}\u{11a32}\u{11a33}\u{11a38}\u{11a39}\u{11a3a}\u{11a3b}\u{11a3e}\u{11a3f}\u{11a46}\u{11a47}\u{11a4f}\u{11a50}\u{11a51}\u{11a56}\u{11a57}\u{11a58}\u{11a59}\u{11a5b}\u{11a5c}\u{11a89}\u{11a8a}\u{11a96}\u{11a97}\u{11a98}\u{11a99}\u{11a9a}\u{11aa2}\u{11aa3}\u{11aaf}\u{11ab0}\u{11af8}\u{11af9}\u{11aff}\u{11b00}\u{11b0b}\u{11b60}\u{11b61}\u{11b62}\u{11b64}\u{11b65}\u{11b66}\u{11b67}\u{11b68}\u{11bbf}\u{11bc0}\u{11be1}\u{11be2}\u{11bef}\u{11bf0}\u{11bf9}\u{11bfa}\u{11bff}\u{11c00}\u{11c08}\u{11c09}\u{11c0a}\u{11c2f}\u{11c30}\u{11c3d}\u{11c3e}\u{11c3f}\u{11c40}\u{11c45}\u{11c46}\u{11c4f}\u{11c50}\u{11c6c}\u{11c6d}\u{11c6f}\u{11c70}\u{11c8f}\u{11c90}\u{11ca8}\u{11ca9}\u{11caa}\u{11cb0}\u{11cb1}\u{11cb2}\u{11cb3}\u{11cb4}\u{11cb5}\u{11cff}\u{11d00}\u{11d06}\u{11d07}\u{11d08}\u{11d09}\u{11d0a}\u{11d0b}\u{11d30}\u{11d31}\u{11d45}\u{11d46}\u{11d47}\u{11d4f}\u{11d50}\u{11d59}\u{11d5a}\u{11d5f}\u{11d60}\u{11d65}\u{11d66}\u{11d67}\u{11d68}\u{11d69}\u{11d6a}\u{11d8e}\u{11d8f}\u{11d92}\u{11d93}\u{11d94}\u{11d95}\u{11d96}\u{11d97}\u{11d98}\u{11d99}\u{11d9f}\u{11da0}\u{11da9}\u{11daa}\u{11daf}\u{11db0}\u{11ddb}\u{11ddc}\u{11ddf}\u{11de0}\u{11de9}\u{11dea}\u{11df0}\u{11df2}\u{11edf}\u{11ee0}\u{11ef2}\u{11ef3}\u{11ef4}\u{11ef5}\u{11ef8}\u{11ef9}\u{11f01}\u{11f02}\u{11f10}\u{11f11}\u{11f12}\u{11f35}\u{11f36}\u{11f3d}\u{11f3e}\u{11f3f}\u{11f40}\u{11f42}\u{11f43}\u{11f59}\u{11f5a}\u{11faf}\u{11fb0}\u{11fb1}\u{11fbf}\u{11fc0}\u{11ff1}\u{11ff2}\u{11ffe}\u{11fff}\u{12399}\u{1239a}\u{123ff}\u{12400}\u{12543}\u{12544}\u{1254f}\u{12687}\u{12f8f}\u{12f90}\u{12ff2}\u{12ff3}\u{12fff}\u{13000}\u{1342f}\u{13430}\u{13440}\u{13441}\u{13446}\u{13447}\u{1345f}\u{13460}\u{143fa}\u{143fb}\u{143ff}\u{14400}\u{14646}\u{14647}\u{160ff}\u{16100}\u{1611d}\u{1611e}\u{16129}\u{1612a}\u{1612c}\u{1612d}\u{1612f}\u{16130}\u{16139}\u{1613a}\u{167ff}\u{16800}\u{16a38}\u{16a39}\u{16a3f}\u{16a40}\u{16a5e}\u{16a5f}\u{16a60}\u{16a69}\u{16a6a}\u{16a6d}\u{16a6e}\u{16abe}\u{16abf}\u{16ac0}\u{16ac9}\u{16aca}\u{16acf}\u{16ad0}\u{16aed}\u{16aee}\u{16af4}\u{16af5}\u{16af6}\u{16aff}\u{16b00}\u{16b2f}\u{16b30}\u{16b36}\u{16b37}\u{16b45}\u{16b46}\u{16b4f}\u{16b50}\u{16b59}\u{16b5a}\u{16b5b}\u{16b61}\u{16b62}\u{16b63}\u{16b77}\u{16b78}\u{16b7c}\u{16b7d}\u{16b8f}\u{16b90}\u{16d3f}\u{16d40}\u{16d79}\u{16d7a}\u{16e3f}\u{16e40}\u{16e9a}\u{16e9b}\u{16e9f}\u{16ea0}\u{16eb8}\u{16eb9}\u{16eba}\u{16ebb}\u{16ed3}\u{16ed4}\u{16eff}\u{16f00}\u{16f4a}\u{16f4b}\u{16f4f}\u{16f50}\u{16f87}\u{16f88}\u{16f92}\u{16f93}\u{16f9f}\u{16fa0}\u{16fdf}\u{16fe0}\u{16fe3}\u{16fe4}\u{16ff1}\u{16ff2}\u{16ff6}\u{16ff7}\u{16fff}\u{17000}\u{18cdb}\u{18cfe}\u{18cff}\u{18d21}\u{18d7f}\u{18d80}\u{18df2}\u{18df3}\u{18dff}\u{19192}\u{1919f}\u{191d3}\u{1afef}\u{1aff0}\u{1aff3}\u{1aff4}\u{1aff5}\u{1affb}\u{1affc}\u{1affd}\u{1affe}\u{1afff}\u{1b000}\u{1b129}\u{1b131}\u{1b132}\u{1b133}\u{1b14f}\u{1b150}\u{1b152}\u{1b153}\u{1b154}\u{1b155}\u{1b156}\u{1b163}\u{1b164}\u{1b169}\u{1b16f}\u{1b170}\u{1b2fb}\u{1b2fc}\u{1bbff}\u{1bc00}\u{1bc6a}\u{1bc6b}\u{1bc6f}\u{1bc70}\u{1bc7c}\u{1bc7d}\u{1bc7f}\u{1bc80}\u{1bc88}\u{1bc89}\u{1bc8f}\u{1bc90}\u{1bc99}\u{1bc9a}\u{1bc9b}\u{1bc9c}\u{1bc9d}\u{1bc9e}\u{1bc9f}\u{1bca0}\u{1cbff}\u{1cc00}\u{1ccfc}\u{1ccfd}\u{1ccff}\u{1cd00}\u{1ceb3}\u{1ceb4}\u{1ceb9}\u{1ceba}\u{1ced0}\u{1ced1}\u{1ced5}\u{1cedc}\u{1cefe}\u{1cf4f}\u{1cf50}\u{1cfc3}\u{1cfc4}\u{1cfff}\u{1d000}\u{1d0f5}\u{1d0f6}\u{1d0ff}\u{1d100}\u{1d126}\u{1d127}\u{1d128}\u{1d129}\u{1d164}\u{1d165}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d241}\u{1d242}\u{1d244}\u{1d245}\u{1d246}\u{1d252}\u{1d25b}\u{1d25c}\u{1d25f}\u{1d280}\u{1d2bf}\u{1d2c0}\u{1d2d3}\u{1d2d4}\u{1d2df}\u{1d2e0}\u{1d2f3}\u{1d2f4}\u{1d2ff}\u{1d300}\u{1d356}\u{1d357}\u{1d35f}\u{1d360}\u{1d378}\u{1d379}\u{1d3ff}\u{1d400}\u{1d454}\u{1d455}\u{1d456}\u{1d49c}\u{1d49d}\u{1d49e}\u{1d49f}\u{1d4a0}\u{1d4a1}\u{1d4a2}\u{1d4a3}\u{1d4a4}\u{1d4a5}\u{1d4a6}\u{1d4a7}\u{1d4a8}\u{1d4a9}\u{1d4ac}\u{1d4ad}\u{1d4ae}\u{1d4b9}\u{1d4ba}\u{1d4bb}\u{1d4bc}\u{1d4bd}\u{1d4c3}\u{1d4c4}\u{1d4c5}\u{1d505}\u{1d506}\u{1d507}\u{1d50a}\u{1d50b}\u{1d50c}\u{1d50d}\u{1d514}\u{1d515}\u{1d516}\u{1d51c}\u{1d51d}\u{1d51e}\u{1d539}\u{1d53a}\u{1d53b}\u{1d53e}\u{1d53f}\u{1d540}\u{1d544}\u{1d545}\u{1d546}\u{1d547}\u{1d549}\u{1d54a}\u{1d550}\u{1d551}\u{1d552}\u{1d6a7}\u{1d6a8}\u{1d7cb}\u{1d7cc}\u{1d7cd}\u{1d7ce}\u{1d9ff}\u{1da00}\u{1da36}\u{1da37}\u{1da3a}\u{1da3b}\u{1da6c}\u{1da6d}\u{1da74}\u{1da75}\u{1da76}\u{1da83}\u{1da84}\u{1da85}\u{1da8b}\u{1da8c}\u{1daff}\u{1db1d}\u{1deff}\u{1df00}\u{1df82}\u{1df8f}\u{1df97}\u{1dfcc}\u{1e000}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e0ff}\u{1e100}\u{1e12c}\u{1e12d}\u{1e136}\u{1e137}\u{1e13d}\u{1e13e}\u{1e13f}\u{1e140}\u{1e149}\u{1e14a}\u{1e14d}\u{1e14e}\u{1e14f}\u{1e150}\u{1e28f}\u{1e290}\u{1e2ad}\u{1e2ae}\u{1e2bf}\u{1e2c0}\u{1e2eb}\u{1e2ec}\u{1e2ef}\u{1e2f0}\u{1e2f9}\u{1e2fa}\u{1e2fe}\u{1e2ff}\u{1e300}\u{1e4cf}\u{1e4d0}\u{1e4eb}\u{1e4ec}\u{1e4ef}\u{1e4f0}\u{1e4f9}\u{1e4fa}\u{1e5cf}\u{1e5d0}\u{1e5ed}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e5fa}\u{1e5fb}\u{1e5fe}\u{1e5ff}\u{1e600}\u{1e6bf}\u{1e6c0}\u{1e6de}\u{1e6df}\u{1e6e0}\u{1e6e2}\u{1e6e3}\u{1e6e4}\u{1e6e5}\u{1e6e6}\u{1e6e7}\u{1e6ed}\u{1e6ee}\u{1e6ef}\u{1e6f0}\u{1e6f4}\u{1e6f5}\u{1e6fd}\u{1e6fe}\u{1e6ff}\u{1e700}\u{1e7df}\u{1e7e0}\u{1e7e6}\u{1e7e7}\u{1e7e8}\u{1e7eb}\u{1e7ec}\u{1e7ed}\u{1e7ee}\u{1e7ef}\u{1e7f0}\u{1e7fe}\u{1e7ff}\u{1e800}\u{1e8c4}\u{1e8c5}\u{1e8c6}\u{1e8c7}\u{1e8cf}\u{1e8d0}\u{1e8ff}\u{1e900}\u{1e943}\u{1e944}\u{1e94a}\u{1e94b}\u{1e94c}\u{1e94f}\u{1e950}\u{1e959}\u{1e95a}\u{1e95d}\u{1e95e}\u{1e95f}\u{1e960}\u{1ec70}\u{1ec71}\u{1ecb4}\u{1ecb5}\u{1ed00}\u{1ed01}\u{1ed3d}\u{1ed3e}\u{1edff}\u{1ee00}\u{1ee03}\u{1ee04}\u{1ee05}\u{1ee1f}\u{1ee20}\u{1ee21}\u{1ee22}\u{1ee23}\u{1ee24}\u{1ee25}\u{1ee26}\u{1ee27}\u{1ee28}\u{1ee29}\u{1ee32}\u{1ee33}\u{1ee34}\u{1ee37}\u{1ee38}\u{1ee39}\u{1ee3a}\u{1ee3b}\u{1ee3c}\u{1ee41}\u{1ee42}\u{1ee43}\u{1ee46}\u{1ee47}\u{1ee48}\u{1ee49}\u{1ee4a}\u{1ee4b}\u{1ee4c}\u{1ee4d}\u{1ee4f}\u{1ee50}\u{1ee51}\u{1ee52}\u{1ee53}\u{1ee54}\u{1ee55}\u{1ee56}\u{1ee57}\u{1ee58}\u{1ee59}\u{1ee5a}\u{1ee5b}\u{1ee5c}\u{1ee5d}\u{1ee5e}\u{1ee5f}\u{1ee60}\u{1ee61}\u{1ee62}\u{1ee63}\u{1ee64}\u{1ee65}\u{1ee66}\u{1ee67}\u{1ee6a}\u{1ee6b}\u{1ee6c}\u{1ee72}\u{1ee73}\u{1ee74}\u{1ee77}\u{1ee78}\u{1ee79}\u{1ee7c}\u{1ee7d}\u{1ee7e}\u{1ee7f}\u{1ee80}\u{1ee89}\u{1ee8a}\u{1ee8b}\u{1ee9b}\u{1ee9c}\u{1eea0}\u{1eea1}\u{1eea3}\u{1eea4}\u{1eea5}\u{1eea9}\u{1eeaa}\u{1eeab}\u{1eebb}\u{1eebc}\u{1eeef}\u{1eef0}\u{1eef1}\u{1eef2}\u{1efff}\u{1f000}\u{1f02b}\u{1f02c}\u{1f02f}\u{1f030}\u{1f093}\u{1f094}\u{1f09f}\u{1f0a0}\u{1f0ae}\u{1f0af}\u{1f0b0}\u{1f0b1}\u{1f0bf}\u{1f0c0}\u{1f0c1}\u{1f0cf}\u{1f0d0}\u{1f0d1}\u{1f0f5}\u{1f0f6}\u{1f0ff}\u{1f100}\u{1f1af}\u{1f1e5}\u{1f1e6}\u{1f202}\u{1f203}\u{1f20f}\u{1f210}\u{1f23b}\u{1f23c}\u{1f23f}\u{1f240}\u{1f248}\u{1f249}\u{1f24f}\u{1f250}\u{1f251}\u{1f252}\u{1f25f}\u{1f260}\u{1f265}\u{1f266}\u{1f2ff}\u{1f300}\u{1f6da}\u{1f6db}\u{1f6dc}\u{1f6ec}\u{1f6ed}\u{1f6ef}\u{1f6f0}\u{1f6fc}\u{1f6fd}\u{1f6ff}\u{1f700}\u{1f7dc}\u{1f7df}\u{1f7e0}\u{1f7eb}\u{1f7ec}\u{1f7ef}\u{1f7f0}\u{1f80b}\u{1f80c}\u{1f80f}\u{1f810}\u{1f847}\u{1f848}\u{1f84f}\u{1f850}\u{1f859}\u{1f85a}\u{1f85f}\u{1f860}\u{1f887}\u{1f888}\u{1f88f}\u{1f890}\u{1f8ad}\u{1f8ae}\u{1f8af}\u{1f8b0}\u{1f8bb}\u{1f8bc}\u{1f8bf}\u{1f8c0}\u{1f8c1}\u{1f8c2}\u{1f8cf}\u{1f8d0}\u{1f8d8}\u{1f8d9}\u{1f8ff}\u{1f900}\u{1fa57}\u{1fa58}\u{1fa5f}\u{1fa60}\u{1fa6d}\u{1fa6e}\u{1fa6f}\u{1fa70}\u{1fa7c}\u{1fa7d}\u{1fa7f}\u{1fa80}\u{1fac6}\u{1fac7}\u{1fac8}\u{1fac9}\u{1facb}\u{1fade}\u{1fadf}\u{1faec}\u{1faee}\u{1faef}\u{1fafb}\u{1faff}\u{1fb00}\u{1fb92}\u{1fb93}\u{1fb94}\u{1fbfa}\u{1fbfb}\u{1ffff}\u{20000}\u{2a6df}\u{2a6e0}\u{2a6ff}\u{2a700}\u{2b81f}\u{2b820}\u{2cead}\u{2ceae}\u{2ceaf}\u{2ceb0}\u{2ebe0}\u{2ebe1}\u{2ebef}\u{2ebf0}\u{2ee5d}\u{2ee5e}\u{2f7ff}\u{2f800}\u{2fa1d}\u{2fa1e}\u{2ffff}\u{30000}\u{3134a}\u{3134b}\u{3134f}\u{31350}\u{33479}\u{3347a}\u{3cfff}\u{3fc40}\u{feff}ſK\n",
                "X\r\n\u{41}\u{2028}\u{42}\u{2029}",
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
fn unicode_grapheme_base_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Gr_Base}}$/i{mode},n=/^\P{{Gr_Base}}$/i{mode};'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Gr_Base}}K]/i{mode},n=/[\P{{Gr_Base}}K]/i{mode},x=/[^\P{{Gr_Base}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Grapheme_Base}}/{mode},b=/\p{{Gr_Base}}/i{mode},n=/\P{{Gr_Base}}/i{mode};a.test('\u0041')&&a.test('\u0042')&&b.test('\u0041')&&!a.test('\u200D')&&b.test('\u005f')&&b.test('ſ')&&b.test('K')&&!n.test('\u0041')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Grapheme_Base}}/{mode},s=/\s/{mode};p.test('\u0041')&&p.test('\u0042')&&!s.test('\u0041')&&!s.test('\u0042')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Gr_Base}}K]/i{mode},b=/[^\P{{Grapheme_Base}}K]/i{mode},c=/[\p{{Gr_Base}}\P{{Grapheme_Base}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0041')&&a.test('ſ')&&b.test('\u0042')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Gr_Base}}$/{mode},b=/^\P{{Gr_Base}}$/{mode};a.test('\u0041')&&!a.test('\u0042\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_grapheme_base_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Gr_Base}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD835\uDC22');m.index===0&&m[0]==='\uD835\uDC22'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Gr_Base}}K]))$/dim{mode}g,m=r.exec('\u200D\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Gr_Base}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Gr_Base}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===2&&m[0]==='\u0041'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Gr_Base}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0041')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_grapheme_base_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0042\uDB40\uDC01\u0041\u200D'.matchAll(/(?<x>\P{{Gr_Base}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u200D'"
        ));
        check(&format!(
            r"'\u0042\u0041\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Grapheme_Base}})/{mode}g,'<$1:$<x>>')==='<\u0042:\u0042><\u0041:\u0041>\uDB40\uDC01\u200D'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u0042\u0041\u200D'.split(/(\p{{Gr_Base}})/{mode}).join('|')==='\uDB40\uDC01|\u0042||\u0041|\u200D'&&'\uDB40\uDC01\u0041'.search(/\p{{Gr_Base}}/{mode})===2&&'\u0042\u0041\uDB40\uDC01'.match(/\p{{Gr_Base}}/{mode}g).join('')==='\u0042\u0041'"
        ));
        check(&format!(
            r"let t='';let x='\u0041\u0042\uDB40\uDC01\u0041'.replace(/(\p{{Gr_Base}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_grapheme_base_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Grapheme_Base})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Grapheme_Base})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0041').groups.x==='\u0041'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Gr_Base}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Gr_Base}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Gr_Base}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_grapheme_base_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Gr_Base}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u0041'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0041'&&m[3000]==='\\u0041'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_grapheme_base_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    // Keep plan construction and search-input allocation in separate Scripts.
    // Each Script receives the same explicitly opted-in work budget.
    realm.eval("let r=/\\p{Gr_Base}/ug,marker=0").unwrap();
    realm
        .eval("let text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/\p{Script_Extensions=Latin}/u.exec('A')",
        r"/[\p{Gr_Base}&&A]/v.exec('A')",
        r"/[\p{Gr_Base}--A]/v.exec('A')",
        r"/\p{Gr_Base}+b/u.exec('AA')",
        r"/\p{Gr_Base}A/v.exec('AA')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn unicode_grapheme_base_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\b(?<x>\p{{Gr_Base}})+\b/d{mode}g,m=r.exec('\uDB40\uDC01\u0041\u0042\u0042\uFEFF');m.index===2&&m[0]==='\u0041\u0042\u0042'&&m.groups.x==='\u0042'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_grapheme_base_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x208fu32, 0x209du32, 0x209eu32, 0x209fu32, 0x20c2u32,
        0x20c3u32, 0x20c4u32, 0x2e60u32, 0x2e61u32, 0x2e63u32, 0xa7ddu32, 0xa7e2u32, 0xab6cu32,
        0xab6du32, 0x107bbu32, 0x107bdu32, 0x107bfu32, 0x10ec9u32, 0x10ecau32, 0x10ed9u32,
        0x10ee3u32, 0x10eeeu32, 0x11b0au32, 0x11df1u32, 0x1246fu32, 0x12475u32, 0x1247au32,
        0x1247fu32, 0x12550u32, 0x125ebu32, 0x12686u32, 0x18cd6u32, 0x18cd8u32, 0x18cdau32,
        0x18d1fu32, 0x18d20u32, 0x18e00u32, 0x18fc8u32, 0x19191u32, 0x191a0u32, 0x191b9u32,
        0x191d2u32, 0x1b123u32, 0x1b125u32, 0x1b128u32, 0x1b168u32, 0x1ced2u32, 0x1ced3u32,
        0x1ced4u32, 0x1ceddu32, 0x1cedeu32, 0x1cedfu32, 0x1cef1u32, 0x1cef7u32, 0x1cefdu32,
        0x1d1ebu32, 0x1d1f5u32, 0x1d1ffu32, 0x1d253u32, 0x1d256u32, 0x1d25au32, 0x1d25du32,
        0x1d25eu32, 0x1d260u32, 0x1d26fu32, 0x1d27fu32, 0x1d6a6u32, 0x1db00u32, 0x1db0eu32,
        0x1db1cu32, 0x1df1fu32, 0x1df21u32, 0x1df24u32, 0x1df2bu32, 0x1df40u32, 0x1df48u32,
        0x1df4au32, 0x1df4du32, 0x1df51u32, 0x1df56u32, 0x1df68u32, 0x1df6au32, 0x1df6cu32,
        0x1df6eu32, 0x1df72u32, 0x1df74u32, 0x1df76u32, 0x1df78u32, 0x1df7au32, 0x1df7cu32,
        0x1df7eu32, 0x1df81u32, 0x1df90u32, 0x1df93u32, 0x1df95u32, 0x1df96u32, 0x1dfcdu32,
        0x1dfe6u32, 0x1dfffu32, 0x1f1aeu32, 0x1f6d9u32, 0x1f7dau32, 0x1f7dbu32, 0x1f7f1u32,
        0x1f7f8u32, 0x1f7ffu32, 0x1fa8bu32, 0x1fa8cu32, 0x1fa8du32, 0x1faccu32, 0x1faddu32,
        0x1faebu32, 0x1faf9u32, 0x1fafau32, 0x2b81eu32, 0x3d000u32, 0x3e61fu32, 0x3fc3fu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Grapheme_Base", "Gr_Base"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 968);
}
