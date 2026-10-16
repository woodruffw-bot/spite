//! ID_Continue and its exact IDC alias over complete Unicode characters.
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
fn unicode_id_continue_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{ID_Continue}",
        r"\P{IDC}",
        r"[^\p{IDC}]",
        r"[\p{IDC}K]",
        r"[\P{ID_Continue}K]",
        r"^(?<x>[\p{IDC}K])$",
        r"(?<x>\P{ID_Continue})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{2f}\u{30}\u{39}\u{3a}\u{40}\u{41}\u{5a}\u{5b}\u{5e}\u{5f}\u{60}\u{61}\u{7a}\u{7b}\u{a9}\u{aa}\u{ab}\u{b4}\u{b5}\u{b6}\u{b7}\u{b8}\u{b9}\u{ba}\u{bb}\u{bf}\u{c0}\u{d6}\u{d7}\u{d8}\u{f6}\u{f7}\u{f8}\u{2c1}\u{2c2}\u{2c5}\u{2c6}\u{2d1}\u{2d2}\u{2df}\u{2e0}\u{2e4}\u{2e5}\u{2eb}\u{2ec}\u{2ed}\u{2ee}\u{2ef}\u{2ff}\u{300}\u{374}\u{375}\u{376}\u{377}\u{378}\u{379}\u{37a}\u{37d}\u{37e}\u{37f}\u{380}\u{385}\u{386}\u{38a}\u{38b}\u{38c}\u{38d}\u{38e}\u{3a1}\u{3a2}\u{3a3}\u{3f5}\u{3f6}\u{3f7}\u{481}\u{482}\u{483}\u{487}\u{488}\u{489}\u{48a}\u{52f}\u{530}\u{531}\u{556}\u{557}\u{559}\u{55a}\u{55f}\u{560}\u{588}\u{589}\u{58a}\u{58d}\u{590}\u{591}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5ca}\u{5cf}\u{5d0}\u{5ea}\u{5eb}\u{5ee}\u{5ef}\u{5f2}\u{5f3}\u{60f}\u{610}\u{61a}\u{61b}\u{61f}\u{620}\u{669}\u{66a}\u{66d}\u{66e}\u{6d3}\u{6d4}\u{6d5}\u{6dc}\u{6dd}\u{6de}\u{6df}\u{6e8}\u{6e9}\u{6ea}\u{6fc}\u{6fd}\u{6fe}\u{6ff}\u{700}\u{70f}\u{710}\u{74a}\u{74b}\u{74c}\u{74d}\u{7b1}\u{7b2}\u{7bf}\u{7c0}\u{7f5}\u{7f6}\u{7f9}\u{7fa}\u{7fb}\u{7fc}\u{7fd}\u{7fe}\u{7ff}\u{800}\u{82d}\u{82e}\u{83f}\u{840}\u{85b}\u{85c}\u{85f}\u{860}\u{86a}\u{86b}\u{86f}\u{870}\u{887}\u{888}\u{889}\u{88f}\u{890}\u{896}\u{897}\u{8e1}\u{8e2}\u{8e3}\u{963}\u{964}\u{965}\u{966}\u{96f}\u{970}\u{971}\u{983}\u{984}\u{985}\u{98c}\u{98d}\u{98e}\u{98f}\u{990}\u{991}\u{992}\u{993}\u{9a8}\u{9a9}\u{9aa}\u{9b0}\u{9b1}\u{9b2}\u{9b3}\u{9b5}\u{9b6}\u{9b9}\u{9ba}\u{9bb}\u{9bc}\u{9c4}\u{9c5}\u{9c6}\u{9c7}\u{9c8}\u{9c9}\u{9ca}\u{9cb}\u{9ce}\u{9cf}\u{9d6}\u{9d7}\u{9d8}\u{9db}\u{9dc}\u{9dd}\u{9de}\u{9df}\u{9e3}\u{9e4}\u{9e5}\u{9e6}\u{9f1}\u{9f2}\u{9fb}\u{9fc}\u{9fd}\u{9fe}\u{9ff}\u{a00}\u{a01}\u{a03}\u{a04}\u{a05}\u{a0a}\u{a0b}\u{a0e}\u{a0f}\u{a10}\u{a11}\u{a12}\u{a13}\u{a28}\u{a29}\u{a2a}\u{a30}\u{a31}\u{a32}\u{a33}\u{a34}\u{a35}\u{a36}\u{a37}\u{a38}\u{a39}\u{a3a}\u{a3b}\u{a3c}\u{a3d}\u{a3e}\u{a42}\u{a43}\u{a46}\u{a47}\u{a48}\u{a49}\u{a4a}\u{a4b}\u{a4d}\u{a4e}\u{a50}\u{a51}\u{a52}\u{a58}\u{a59}\u{a5c}\u{a5d}\u{a5e}\u{a5f}\u{a65}\u{a66}\u{a75}\u{a76}\u{a80}\u{a81}\u{a83}\u{a84}\u{a85}\u{a8d}\u{a8e}\u{a8f}\u{a91}\u{a92}\u{a93}\u{aa8}\u{aa9}\u{aaa}\u{ab0}\u{ab1}\u{ab2}\u{ab3}\u{ab4}\u{ab5}\u{ab9}\u{aba}\u{abb}\u{abc}\u{ac5}\u{ac6}\u{ac7}\u{ac9}\u{aca}\u{acb}\u{acd}\u{ace}\u{acf}\u{ad0}\u{ad1}\u{adf}\u{ae0}\u{ae3}\u{ae4}\u{ae5}\u{ae6}\u{aef}\u{af0}\u{af8}\u{af9}\u{aff}\u{b00}\u{b01}\u{b03}\u{b04}\u{b05}\u{b0c}\u{b0d}\u{b0e}\u{b0f}\u{b10}\u{b11}\u{b12}\u{b13}\u{b28}\u{b29}\u{b2a}\u{b30}\u{b31}\u{b32}\u{b33}\u{b34}\u{b35}\u{b39}\u{b3a}\u{b3b}\u{b3c}\u{b44}\u{b45}\u{b46}\u{b47}\u{b48}\u{b49}\u{b4a}\u{b4b}\u{b4d}\u{b4e}\u{b52}\u{b57}\u{b58}\u{b5b}\u{b5c}\u{b5d}\u{b5e}\u{b5f}\u{b63}\u{b64}\u{b65}\u{b66}\u{b6f}\u{b70}\u{b71}\u{b72}\u{b81}\u{b82}\u{b83}\u{b84}\u{b85}\u{b8a}\u{b8b}\u{b8d}\u{b8e}\u{b90}\u{b91}\u{b92}\u{b95}\u{b96}\u{b98}\u{b99}\u{b9a}\u{b9b}\u{b9c}\u{b9d}\u{b9e}\u{b9f}\u{ba0}\u{ba2}\u{ba3}\u{ba4}\u{ba5}\u{ba7}\u{ba8}\u{baa}\u{bab}\u{bad}\u{bae}\u{bb9}\u{bba}\u{bbd}\u{bbe}\u{bc2}\u{bc3}\u{bc5}\u{bc6}\u{bc8}\u{bc9}\u{bca}\u{bcd}\u{bce}\u{bcf}\u{bd0}\u{bd1}\u{bd6}\u{bd7}\u{bd8}\u{be5}\u{be6}\u{bef}\u{bf0}\u{bff}\u{c00}\u{c0c}\u{c0d}\u{c0e}\u{c10}\u{c11}\u{c12}\u{c28}\u{c29}\u{c2a}\u{c39}\u{c3a}\u{c3b}\u{c3c}\u{c44}\u{c45}\u{c46}\u{c48}\u{c49}\u{c4a}\u{c4d}\u{c4e}\u{c54}\u{c55}\u{c56}\u{c57}\u{c58}\u{c5a}\u{c5b}\u{c5c}\u{c5d}\u{c5e}\u{c5f}\u{c60}\u{c63}\u{c64}\u{c65}\u{c66}\u{c6f}\u{c70}\u{c7f}\u{c80}\u{c83}\u{c84}\u{c85}\u{c8c}\u{c8d}\u{c8e}\u{c90}\u{c91}\u{c92}\u{ca8}\u{ca9}\u{caa}\u{cb3}\u{cb4}\u{cb5}\u{cb9}\u{cba}\u{cbb}\u{cbc}\u{cc4}\u{cc5}\u{cc6}\u{cc8}\u{cc9}\u{cca}\u{ccd}\u{cce}\u{cd4}\u{cd5}\u{cd6}\u{cd7}\u{cdb}\u{cdc}\u{cde}\u{cdf}\u{ce0}\u{ce3}\u{ce4}\u{ce5}\u{ce6}\u{cef}\u{cf0}\u{cf1}\u{cf3}\u{cf4}\u{cff}\u{d00}\u{d0c}\u{d0d}\u{d0e}\u{d10}\u{d11}\u{d12}\u{d44}\u{d45}\u{d46}\u{d48}\u{d49}\u{d4a}\u{d4e}\u{d4f}\u{d53}\u{d54}\u{d57}\u{d58}\u{d5e}\u{d5f}\u{d63}\u{d64}\u{d65}\u{d66}\u{d6f}\u{d70}\u{d79}\u{d7a}\u{d7f}\u{d80}\u{d81}\u{d83}\u{d84}\u{d85}\u{d96}\u{d97}\u{d99}\u{d9a}\u{db1}\u{db2}\u{db3}\u{dbb}\u{dbc}\u{dbd}\u{dbe}\u{dbf}\u{dc0}\u{dc6}\u{dc7}\u{dc9}\u{dca}\u{dcb}\u{dce}\u{dcf}\u{dd4}\u{dd5}\u{dd6}\u{dd7}\u{dd8}\u{ddf}\u{de0}\u{de5}\u{de6}\u{def}\u{df0}\u{df1}\u{df2}\u{df3}\u{df4}\u{e00}\u{e01}\u{e3a}\u{e3b}\u{e3f}\u{e40}\u{e4e}\u{e4f}\u{e50}\u{e59}\u{e5a}\u{e80}\u{e81}\u{e82}\u{e83}\u{e84}\u{e85}\u{e86}\u{e8a}\u{e8b}\u{e8c}\u{ea3}\u{ea4}\u{ea5}\u{ea6}\u{ea7}\u{ebd}\u{ebe}\u{ebf}\u{ec0}\u{ec4}\u{ec5}\u{ec6}\u{ec7}\u{ec8}\u{ece}\u{ecf}\u{ed0}\u{ed9}\u{eda}\u{edb}\u{edc}\u{edf}\u{ee0}\u{eff}\u{f00}\u{f01}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f1f}\u{f20}\u{f29}\u{f2a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f3d}\u{f3e}\u{f47}\u{f48}\u{f49}\u{f6c}\u{f6d}\u{f70}\u{f71}\u{f84}\u{f85}\u{f86}\u{f97}\u{f98}\u{f99}\u{fbc}\u{fbd}\u{fc5}\u{fc6}\u{fc7}\u{fff}\u{1000}\u{1049}\u{104a}\u{104f}\u{1050}\u{109d}\u{109e}\u{109f}\u{10a0}\u{10c5}\u{10c6}\u{10c7}\u{10c8}\u{10cc}\u{10cd}\u{10ce}\u{10cf}\u{10d0}\u{10fa}\u{10fb}\u{10fc}\u{1248}\u{1249}\u{124a}\u{124d}\u{124e}\u{124f}\u{1250}\u{1256}\u{1257}\u{1258}\u{1259}\u{125a}\u{125d}\u{125e}\u{125f}\u{1260}\u{1288}\u{1289}\u{128a}\u{128d}\u{128e}\u{128f}\u{1290}\u{12b0}\u{12b1}\u{12b2}\u{12b5}\u{12b6}\u{12b7}\u{12b8}\u{12be}\u{12bf}\u{12c0}\u{12c1}\u{12c2}\u{12c5}\u{12c6}\u{12c7}\u{12c8}\u{12d6}\u{12d7}\u{12d8}\u{1310}\u{1311}\u{1312}\u{1315}\u{1316}\u{1317}\u{1318}\u{135a}\u{135b}\u{135c}\u{135d}\u{135f}\u{1360}\u{1368}\u{1369}\u{1371}\u{1372}\u{137f}\u{1380}\u{138f}\u{1390}\u{139f}\u{13a0}\u{13f5}\u{13f6}\u{13f7}\u{13f8}\u{13fd}\u{13fe}\u{1400}\u{1401}\u{166c}\u{166d}\u{166e}\u{166f}\u{167f}\u{1680}\u{1681}\u{169a}\u{169b}\u{169f}\u{16a0}\u{16ea}\u{16eb}\u{16ed}\u{16ee}\u{16f8}\u{16f9}\u{16ff}\u{1700}\u{1715}\u{1716}\u{171e}\u{171f}\u{1734}\u{1735}\u{173f}\u{1740}\u{1753}\u{1754}\u{175f}\u{1760}\u{176c}\u{176d}\u{176e}\u{1770}\u{1771}\u{1772}\u{1773}\u{1774}\u{177f}\u{1780}\u{17d3}\u{17d4}\u{17d6}\u{17d7}\u{17d8}\u{17db}\u{17dc}\u{17dd}\u{17de}\u{17df}\u{17e0}\u{17e9}\u{17ea}\u{180a}\u{180b}\u{180d}\u{180e}\u{180f}\u{1819}\u{181a}\u{181f}\u{1820}\u{1878}\u{1879}\u{187f}\u{1880}\u{18aa}\u{18ab}\u{18af}\u{18b0}\u{18f5}\u{18f6}\u{18ff}\u{1900}\u{191e}\u{191f}\u{1920}\u{192b}\u{192c}\u{192f}\u{1930}\u{193b}\u{193c}\u{1945}\u{1946}\u{196d}\u{196e}\u{196f}\u{1970}\u{1974}\u{1975}\u{197f}\u{1980}\u{19ab}\u{19ac}\u{19af}\u{19b0}\u{19c9}\u{19ca}\u{19cf}\u{19d0}\u{19da}\u{19db}\u{19ff}\u{1a00}\u{1a1b}\u{1a1c}\u{1a1f}\u{1a20}\u{1a5e}\u{1a5f}\u{1a60}\u{1a7c}\u{1a7d}\u{1a7e}\u{1a7f}\u{1a89}\u{1a8a}\u{1a8f}\u{1a90}\u{1a99}\u{1a9a}\u{1aa6}\u{1aa7}\u{1aa8}\u{1aaf}\u{1ab0}\u{1abd}\u{1abe}\u{1abf}\u{1af1}\u{1aff}\u{1b00}\u{1b4c}\u{1b4d}\u{1b4f}\u{1b50}\u{1b59}\u{1b5a}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1b7f}\u{1b80}\u{1bf3}\u{1bf4}\u{1bff}\u{1c00}\u{1c37}\u{1c38}\u{1c3f}\u{1c40}\u{1c49}\u{1c4a}\u{1c4c}\u{1c4d}\u{1c7d}\u{1c7e}\u{1c7f}\u{1c80}\u{1c8a}\u{1c8b}\u{1c8f}\u{1c90}\u{1cba}\u{1cbb}\u{1cbc}\u{1cbd}\u{1cbf}\u{1cc0}\u{1ccf}\u{1cd0}\u{1cd2}\u{1cd3}\u{1cd4}\u{1cfa}\u{1cfb}\u{1cff}\u{1d00}\u{1f15}\u{1f16}\u{1f17}\u{1f18}\u{1f1d}\u{1f1e}\u{1f1f}\u{1f20}\u{1f45}\u{1f46}\u{1f47}\u{1f48}\u{1f4d}\u{1f4e}\u{1f4f}\u{1f50}\u{1f57}\u{1f58}\u{1f59}\u{1f5a}\u{1f5b}\u{1f5c}\u{1f5d}\u{1f5e}\u{1f5f}\u{1f7d}\u{1f7e}\u{1f7f}\u{1f80}\u{1fb4}\u{1fb5}\u{1fb6}\u{1fbc}\u{1fbd}\u{1fbe}\u{1fbf}\u{1fc1}\u{1fc2}\u{1fc4}\u{1fc5}\u{1fc6}\u{1fcc}\u{1fcd}\u{1fcf}\u{1fd0}\u{1fd3}\u{1fd4}\u{1fd5}\u{1fd6}\u{1fdb}\u{1fdc}\u{1fdf}\u{1fe0}\u{1fec}\u{1fed}\u{1ff1}\u{1ff2}\u{1ff4}\u{1ff5}\u{1ff6}\u{1ffc}\u{1ffd}\u{200b}\u{200c}\u{200d}\u{41}\u{203e}\u{203f}\u{2040}\u{2041}\u{2053}\u{2054}\u{2055}\u{2070}\u{2071}\u{2072}\u{207e}\u{207f}\u{2080}\u{208e}\u{20a0}\u{20cf}\u{20d0}\u{20dc}\u{20dd}\u{20e0}\u{20e1}\u{20e2}\u{20e4}\u{20e5}\u{20f0}\u{20f1}\u{2101}\u{2102}\u{2103}\u{2106}\u{2107}\u{2108}\u{2109}\u{210a}\u{2113}\u{2114}\u{2115}\u{2116}\u{2117}\u{2118}\u{211d}\u{211e}\u{2123}\u{2124}\u{2125}\u{2126}\u{2127}\u{2128}\u{2129}\u{212a}\u{2139}\u{213a}\u{213b}\u{213c}\u{213f}\u{2140}\u{2144}\u{2145}\u{2149}\u{214a}\u{214d}\u{214e}\u{214f}\u{215f}\u{2160}\u{2188}\u{2189}\u{2bff}\u{2c00}\u{2ce4}\u{2ce5}\u{2cea}\u{2ceb}\u{2cf3}\u{2cf4}\u{2cff}\u{2d00}\u{2d25}\u{2d26}\u{2d27}\u{2d28}\u{2d2c}\u{2d2d}\u{2d2e}\u{2d2f}\u{2d30}\u{2d67}\u{2d68}\u{2d6e}\u{2d6f}\u{2d70}\u{2d7e}\u{2d7f}\u{2d96}\u{2d97}\u{2d9f}\u{2da0}\u{2da6}\u{2da7}\u{2da8}\u{2dae}\u{2daf}\u{2db0}\u{2db6}\u{2db7}\u{2db8}\u{2dbe}\u{2dbf}\u{2dc0}\u{2dc6}\u{2dc7}\u{2dc8}\u{2dce}\u{2dcf}\u{2dd0}\u{2dd6}\u{2dd7}\u{2dd8}\u{2dde}\u{2ddf}\u{2de0}\u{2dff}\u{2e00}\u{3004}\u{3005}\u{3007}\u{3008}\u{3020}\u{3021}\u{302f}\u{3030}\u{3031}\u{3035}\u{3036}\u{3037}\u{3038}\u{303c}\u{303d}\u{3040}\u{3041}\u{3096}\u{3097}\u{3098}\u{3099}\u{309f}\u{30a0}\u{30a1}\u{30ff}\u{3100}\u{3104}\u{3105}\u{312f}\u{3130}\u{3131}\u{318e}\u{318f}\u{319f}\u{31a0}\u{31bf}\u{31c0}\u{31ef}\u{31f0}\u{31ff}\u{3200}\u{33ff}\u{3400}\u{4dbf}\u{4dc0}\u{4dff}\u{4e00}\u{a48c}\u{a48d}\u{a4cf}\u{a4d0}\u{a4fd}\u{a4fe}\u{a4ff}\u{a500}\u{a60c}\u{a60d}\u{a60f}\u{a610}\u{a62b}\u{a62c}\u{a63f}\u{a640}\u{a66f}\u{a670}\u{a673}\u{a674}\u{a67d}\u{a67e}\u{a67f}\u{a6f1}\u{a6f2}\u{a716}\u{a717}\u{a71f}\u{a720}\u{a721}\u{a722}\u{a788}\u{a789}\u{a78a}\u{a78b}\u{a7de}\u{a7e1}\u{a7e3}\u{a7f0}\u{a7f1}\u{a827}\u{a828}\u{a82b}\u{a82c}\u{a82d}\u{a83f}\u{a840}\u{a873}\u{a874}\u{a87f}\u{a880}\u{a8c5}\u{a8c6}\u{a8cf}\u{a8d0}\u{a8d9}\u{a8da}\u{a8df}\u{a8e0}\u{a8f7}\u{a8f8}\u{a8fa}\u{a8fb}\u{a8fc}\u{a8fd}\u{a92d}\u{a92e}\u{a92f}\u{a930}\u{a953}\u{a954}\u{a95f}\u{a960}\u{a97c}\u{a97d}\u{a97f}\u{a980}\u{a9c0}\u{a9c1}\u{a9ce}\u{a9cf}\u{a9d9}\u{a9da}\u{a9df}\u{a9e0}\u{a9fe}\u{a9ff}\u{aa00}\u{aa36}\u{aa37}\u{aa3f}\u{aa40}\u{aa4d}\u{aa4e}\u{aa4f}\u{aa50}\u{aa59}\u{aa5a}\u{aa5f}\u{aa60}\u{aa76}\u{aa77}\u{aa79}\u{aa7a}\u{aac2}\u{aac3}\u{aada}\u{aadb}\u{aadd}\u{aade}\u{aadf}\u{aae0}\u{aaef}\u{aaf0}\u{aaf1}\u{aaf2}\u{aaf6}\u{aaf7}\u{ab00}\u{ab01}\u{ab06}\u{ab07}\u{ab08}\u{ab09}\u{ab0e}\u{ab0f}\u{ab10}\u{ab11}\u{ab16}\u{ab17}\u{ab1f}\u{ab20}\u{ab26}\u{ab27}\u{ab28}\u{ab2e}\u{ab2f}\u{ab30}\u{ab5a}\u{ab5b}\u{ab5c}\u{ab69}\u{ab6a}\u{ab6b}\u{ab6e}\u{ab6f}\u{ab70}\u{abea}\u{abeb}\u{abec}\u{abed}\u{abee}\u{abef}\u{abf0}\u{abf9}\u{abfa}\u{abff}\u{ac00}\u{d7a3}\u{d7a4}\u{d7af}\u{d7b0}\u{d7c6}\u{d7c7}\u{d7ca}\u{d7cb}\u{d7fb}\u{d7fc}\u{f8ff}\u{f900}\u{fa6d}\u{fa6e}\u{fa6f}\u{fa70}\u{fad9}\u{fada}\u{faff}\u{fb00}\u{fb06}\u{fb07}\u{fb12}\u{fb13}\u{fb17}\u{fb18}\u{fb1c}\u{fb1d}\u{fb28}\u{fb29}\u{fb2a}\u{fb36}\u{fb37}\u{fb38}\u{fb3c}\u{fb3d}\u{fb3e}\u{fb3f}\u{fb40}\u{fb41}\u{fb42}\u{fb43}\u{fb44}\u{fb45}\u{fb46}\u{fbb1}\u{fbb2}\u{fbd2}\u{fbd3}\u{fd3d}\u{fd3e}\u{fd4f}\u{fd50}\u{fd8f}\u{fd90}\u{fd91}\u{fd92}\u{fdc7}\u{fdc8}\u{fdef}\u{fdf0}\u{fdfb}\u{fdfc}\u{fdff}\u{fe00}\u{fe0f}\u{fe10}\u{fe1f}\u{fe20}\u{fe2f}\u{fe30}\u{fe32}\u{fe33}\u{fe34}\u{fe35}\u{fe4c}\u{fe4d}\u{fe4f}\u{fe50}\u{fe6f}\u{fe70}\u{fe74}\u{fe75}\u{fe76}\u{fefc}\u{fefd}\u{ff0f}\u{ff10}\u{ff19}\u{ff1a}\u{ff20}\u{ff21}\u{ff3a}\u{ff3b}\u{ff3e}\u{ff3f}\u{ff40}\u{ff41}\u{ff5a}\u{ff5b}\u{ff64}\u{ff65}\u{ffbe}\u{ffbf}\u{ffc1}\u{ffc2}\u{ffc7}\u{ffc8}\u{ffc9}\u{ffca}\u{ffcf}\u{ffd0}\u{ffd1}\u{ffd2}\u{ffd7}\u{ffd8}\u{ffd9}\u{ffda}\u{ffdc}\u{ffdd}\u{ffff}\u{10000}\u{1000b}\u{1000c}\u{1000d}\u{10026}\u{10027}\u{10028}\u{1003a}\u{1003b}\u{1003c}\u{1003d}\u{1003e}\u{1003f}\u{1004d}\u{1004e}\u{1004f}\u{10050}\u{1005d}\u{1005e}\u{1007f}\u{10080}\u{100fa}\u{100fb}\u{1013f}\u{10140}\u{10174}\u{10175}\u{101fc}\u{101fd}\u{101fe}\u{1027f}\u{10280}\u{1029c}\u{1029d}\u{1029f}\u{102a0}\u{102d0}\u{102d1}\u{102df}\u{102e0}\u{102e1}\u{102ff}\u{10300}\u{1031f}\u{10320}\u{1032c}\u{1032d}\u{1034a}\u{1034b}\u{1034f}\u{10350}\u{1037a}\u{1037b}\u{1037f}\u{10380}\u{1039d}\u{1039e}\u{1039f}\u{103a0}\u{103c3}\u{103c4}\u{103c7}\u{103c8}\u{103cf}\u{103d0}\u{103d1}\u{103d5}\u{103d6}\u{103ff}\u{10400}\u{1049d}\u{1049e}\u{1049f}\u{104a0}\u{104a9}\u{104aa}\u{104af}\u{104b0}\u{104d3}\u{104d4}\u{104d7}\u{104d8}\u{104fb}\u{104fc}\u{104ff}\u{10500}\u{10527}\u{10528}\u{1052f}\u{10530}\u{10563}\u{10564}\u{1056f}\u{10570}\u{1057a}\u{1057b}\u{1057c}\u{1058a}\u{1058b}\u{1058c}\u{10592}\u{10593}\u{10594}\u{10595}\u{10596}\u{10597}\u{105a1}\u{105a2}\u{105a3}\u{105b1}\u{105b2}\u{105b3}\u{105b9}\u{105ba}\u{105bb}\u{105bc}\u{105bd}\u{105bf}\u{105c0}\u{105f3}\u{105f4}\u{105ff}\u{10600}\u{10736}\u{10737}\u{1073f}\u{10740}\u{10755}\u{10756}\u{1075f}\u{10760}\u{10767}\u{10768}\u{1077f}\u{10780}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107c0}\u{107ff}\u{10800}\u{10805}\u{10806}\u{10807}\u{10808}\u{10809}\u{1080a}\u{10835}\u{10836}\u{10837}\u{10838}\u{10839}\u{1083b}\u{1083c}\u{1083d}\u{1083e}\u{1083f}\u{10855}\u{10856}\u{1085f}\u{10860}\u{10876}\u{10877}\u{1087f}\u{10880}\u{1089e}\u{1089f}\u{108df}\u{108e0}\u{108f2}\u{108f3}\u{108f4}\u{108f5}\u{108f6}\u{108ff}\u{10900}\u{10915}\u{10916}\u{1091f}\u{10920}\u{10939}\u{1093a}\u{1093f}\u{10940}\u{10959}\u{1095a}\u{1097f}\u{10980}\u{109b7}\u{109b8}\u{109bd}\u{109be}\u{109bf}\u{109c0}\u{109ff}\u{10a00}\u{10a03}\u{10a04}\u{10a05}\u{10a06}\u{10a07}\u{10a0b}\u{10a0c}\u{10a13}\u{10a14}\u{10a15}\u{10a17}\u{10a18}\u{10a19}\u{10a35}\u{10a36}\u{10a37}\u{10a38}\u{10a3a}\u{10a3b}\u{10a3e}\u{10a3f}\u{10a40}\u{10a5f}\u{10a60}\u{10a7c}\u{10a7d}\u{10a7f}\u{10a80}\u{10a9c}\u{10a9d}\u{10abf}\u{10ac0}\u{10ac7}\u{10ac8}\u{10ac9}\u{10ae6}\u{10ae7}\u{10aff}\u{10b00}\u{10b35}\u{10b36}\u{10b3f}\u{10b40}\u{10b55}\u{10b56}\u{10b5f}\u{10b60}\u{10b72}\u{10b73}\u{10b7f}\u{10b80}\u{10b91}\u{10b92}\u{10bff}\u{10c00}\u{10c48}\u{10c49}\u{10c7f}\u{10c80}\u{10cb2}\u{10cb3}\u{10cbf}\u{10cc0}\u{10cf2}\u{10cf3}\u{10cff}\u{10d00}\u{10d27}\u{10d28}\u{10d2f}\u{10d30}\u{10d39}\u{10d3a}\u{10d3f}\u{10d40}\u{10d65}\u{10d66}\u{10d68}\u{10d69}\u{10d6d}\u{10d6e}\u{10d6f}\u{10d85}\u{10d86}\u{10e7f}\u{10e80}\u{10ea9}\u{10eaa}\u{10eab}\u{10eac}\u{10ead}\u{10eaf}\u{10eb0}\u{10eb1}\u{10eb2}\u{10ec1}\u{10ec2}\u{10ec7}\u{10ec8}\u{10eca}\u{10ed0}\u{10ed8}\u{10eef}\u{10f1c}\u{10f1d}\u{10f26}\u{10f27}\u{10f28}\u{10f2f}\u{10f30}\u{10f50}\u{10f51}\u{10f6f}\u{10f70}\u{10f85}\u{10f86}\u{10faf}\u{10fb0}\u{10fc4}\u{10fc5}\u{10fdf}\u{10fe0}\u{10ff6}\u{10ff7}\u{10fff}\u{11000}\u{11046}\u{11047}\u{11065}\u{11066}\u{11075}\u{11076}\u{1107e}\u{1107f}\u{110ba}\u{110bb}\u{110c1}\u{110c2}\u{110c3}\u{110cf}\u{110d0}\u{110e8}\u{110e9}\u{110ef}\u{110f0}\u{110f9}\u{110fa}\u{110ff}\u{11100}\u{11134}\u{11135}\u{11136}\u{1113f}\u{11140}\u{11143}\u{11144}\u{11147}\u{11148}\u{1114f}\u{11150}\u{11173}\u{11174}\u{11175}\u{11176}\u{11177}\u{1117f}\u{11180}\u{111c4}\u{111c5}\u{111c8}\u{111c9}\u{111cc}\u{111cd}\u{111ce}\u{111da}\u{111db}\u{111dc}\u{111dd}\u{111ff}\u{11200}\u{11211}\u{11212}\u{11213}\u{11237}\u{11238}\u{1123d}\u{1123e}\u{11241}\u{11242}\u{1127f}\u{11280}\u{11286}\u{11287}\u{11288}\u{11289}\u{1128a}\u{1128d}\u{1128e}\u{1128f}\u{1129d}\u{1129e}\u{1129f}\u{112a8}\u{112a9}\u{112af}\u{112b0}\u{112ea}\u{112eb}\u{112ef}\u{112f0}\u{112f9}\u{112fa}\u{112ff}\u{11300}\u{11303}\u{11304}\u{11305}\u{1130c}\u{1130d}\u{1130e}\u{1130f}\u{11310}\u{11311}\u{11312}\u{11313}\u{11328}\u{11329}\u{1132a}\u{11330}\u{11331}\u{11332}\u{11333}\u{11334}\u{11335}\u{11339}\u{1133a}\u{1133b}\u{11344}\u{11345}\u{11346}\u{11347}\u{11348}\u{11349}\u{1134a}\u{1134b}\u{1134d}\u{1134e}\u{1134f}\u{11350}\u{11351}\u{11356}\u{11357}\u{11358}\u{1135c}\u{1135d}\u{11363}\u{11364}\u{11365}\u{11366}\u{1136c}\u{1136d}\u{1136f}\u{11370}\u{11374}\u{11375}\u{1137f}\u{11380}\u{11389}\u{1138a}\u{1138b}\u{1138c}\u{1138d}\u{1138e}\u{1138f}\u{11390}\u{113b5}\u{113b6}\u{113b7}\u{113c0}\u{113c1}\u{113c2}\u{113c3}\u{113c4}\u{113c5}\u{113c6}\u{113c7}\u{113ca}\u{113cb}\u{113cc}\u{113d3}\u{113d4}\u{113e0}\u{113e1}\u{113e2}\u{113e3}\u{113ff}\u{11400}\u{1144a}\u{1144b}\u{1144f}\u{11450}\u{11459}\u{1145a}\u{1145d}\u{1145e}\u{11461}\u{11462}\u{1147f}\u{11480}\u{114c5}\u{114c6}\u{114c7}\u{114c8}\u{114cf}\u{114d0}\u{114d9}\u{114da}\u{1157f}\u{11580}\u{115b5}\u{115b6}\u{115b7}\u{115b8}\u{115c0}\u{115c1}\u{115d7}\u{115d8}\u{115dd}\u{115de}\u{115ff}\u{11600}\u{11640}\u{11641}\u{11643}\u{11644}\u{11645}\u{1164f}\u{11650}\u{11659}\u{1165a}\u{1167f}\u{11680}\u{116b8}\u{116b9}\u{116bf}\u{116c0}\u{116c9}\u{116ca}\u{116cf}\u{116d0}\u{116e3}\u{116e4}\u{116ff}\u{11700}\u{1171a}\u{1171b}\u{1171c}\u{1171d}\u{1172b}\u{1172c}\u{1172f}\u{11730}\u{11739}\u{1173a}\u{1173f}\u{11740}\u{11746}\u{11747}\u{117ff}\u{11800}\u{1183a}\u{1183b}\u{1189f}\u{118a0}\u{118e9}\u{118ea}\u{118fe}\u{118ff}\u{11906}\u{11907}\u{11908}\u{11909}\u{1190a}\u{1190b}\u{1190c}\u{11913}\u{11914}\u{11915}\u{11916}\u{11917}\u{11918}\u{11935}\u{11936}\u{11937}\u{11938}\u{11939}\u{1193a}\u{1193b}\u{11943}\u{11944}\u{1194f}\u{11950}\u{11959}\u{1195a}\u{1199f}\u{119a0}\u{119a7}\u{119a8}\u{119a9}\u{119aa}\u{119d7}\u{119d8}\u{119d9}\u{119da}\u{119e1}\u{119e2}\u{119e3}\u{119e4}\u{119e5}\u{119ff}\u{11a00}\u{11a3e}\u{11a3f}\u{11a46}\u{11a47}\u{11a48}\u{11a4f}\u{11a50}\u{11a99}\u{11a9a}\u{11a9c}\u{11a9d}\u{11a9e}\u{11aaf}\u{11ab0}\u{11af8}\u{11af9}\u{11b09}\u{11b0b}\u{11b5f}\u{11b60}\u{11b67}\u{11b68}\u{11bbf}\u{11bc0}\u{11be0}\u{11be1}\u{11bef}\u{11bf0}\u{11bf9}\u{11bfa}\u{11bff}\u{11c00}\u{11c08}\u{11c09}\u{11c0a}\u{11c36}\u{11c37}\u{11c38}\u{11c40}\u{11c41}\u{11c4f}\u{11c50}\u{11c59}\u{11c5a}\u{11c71}\u{11c72}\u{11c8f}\u{11c90}\u{11c91}\u{11c92}\u{11ca7}\u{11ca8}\u{11ca9}\u{11cb6}\u{11cb7}\u{11cff}\u{11d00}\u{11d06}\u{11d07}\u{11d08}\u{11d09}\u{11d0a}\u{11d0b}\u{11d36}\u{11d37}\u{11d39}\u{11d3a}\u{11d3b}\u{11d3c}\u{11d3d}\u{11d3e}\u{11d3f}\u{11d47}\u{11d48}\u{11d4f}\u{11d50}\u{11d59}\u{11d5a}\u{11d5f}\u{11d60}\u{11d65}\u{11d66}\u{11d67}\u{11d68}\u{11d69}\u{11d6a}\u{11d8e}\u{11d8f}\u{11d90}\u{11d91}\u{11d92}\u{11d93}\u{11d98}\u{11d99}\u{11d9f}\u{11da0}\u{11da9}\u{11daa}\u{11daf}\u{11db0}\u{11ddb}\u{11ddc}\u{11ddf}\u{11de0}\u{11de9}\u{11dea}\u{11def}\u{11df2}\u{11edf}\u{11ee0}\u{11ef6}\u{11ef7}\u{11eff}\u{11f00}\u{11f10}\u{11f11}\u{11f12}\u{11f3a}\u{11f3b}\u{11f3d}\u{11f3e}\u{11f42}\u{11f43}\u{11f4f}\u{11f50}\u{11f5a}\u{11f5b}\u{11faf}\u{11fb0}\u{11fb1}\u{11fff}\u{12000}\u{12399}\u{1239a}\u{123ff}\u{12400}\u{12470}\u{12474}\u{12543}\u{12544}\u{1254f}\u{12687}\u{12f8f}\u{12f90}\u{12ff0}\u{12ff1}\u{12fff}\u{13000}\u{1342f}\u{13430}\u{1343f}\u{13440}\u{13455}\u{13456}\u{1345f}\u{13460}\u{143fa}\u{143fb}\u{143ff}\u{14400}\u{14646}\u{14647}\u{160ff}\u{16100}\u{16139}\u{1613a}\u{167ff}\u{16800}\u{16a38}\u{16a39}\u{16a3f}\u{16a40}\u{16a5e}\u{16a5f}\u{16a60}\u{16a69}\u{16a6a}\u{16a6f}\u{16a70}\u{16abe}\u{16abf}\u{16ac0}\u{16ac9}\u{16aca}\u{16acf}\u{16ad0}\u{16aed}\u{16aee}\u{16aef}\u{16af0}\u{16af4}\u{16af5}\u{16aff}\u{16b00}\u{16b36}\u{16b37}\u{16b3f}\u{16b40}\u{16b43}\u{16b44}\u{16b4f}\u{16b50}\u{16b59}\u{16b5a}\u{16b62}\u{16b63}\u{16b77}\u{16b78}\u{16b7c}\u{16b7d}\u{16b8f}\u{16b90}\u{16d3f}\u{16d40}\u{16d6c}\u{16d6d}\u{16d6f}\u{16d70}\u{16d79}\u{16d7a}\u{16e3f}\u{16e40}\u{16e7f}\u{16e80}\u{16e9f}\u{16ea0}\u{16eb8}\u{16eb9}\u{16eba}\u{16ebb}\u{16ed3}\u{16ed4}\u{16eff}\u{16f00}\u{16f4a}\u{16f4b}\u{16f4e}\u{16f4f}\u{16f87}\u{16f88}\u{16f8e}\u{16f8f}\u{16f9f}\u{16fa0}\u{16fdf}\u{16fe0}\u{16fe1}\u{16fe2}\u{16fe3}\u{16fe4}\u{16fe5}\u{16fef}\u{16ff0}\u{16ff6}\u{16ff7}\u{16fff}\u{17000}\u{18cdb}\u{18cfe}\u{18cff}\u{18d21}\u{18d7f}\u{18d80}\u{18df2}\u{18df3}\u{18dff}\u{19192}\u{1919f}\u{191d3}\u{1afef}\u{1aff0}\u{1aff3}\u{1aff4}\u{1aff5}\u{1affb}\u{1affc}\u{1affd}\u{1affe}\u{1afff}\u{1b000}\u{1b129}\u{1b131}\u{1b132}\u{1b133}\u{1b14f}\u{1b150}\u{1b152}\u{1b153}\u{1b154}\u{1b155}\u{1b156}\u{1b163}\u{1b164}\u{1b169}\u{1b16f}\u{1b170}\u{1b2fb}\u{1b2fc}\u{1bbff}\u{1bc00}\u{1bc6a}\u{1bc6b}\u{1bc6f}\u{1bc70}\u{1bc7c}\u{1bc7d}\u{1bc7f}\u{1bc80}\u{1bc88}\u{1bc89}\u{1bc8f}\u{1bc90}\u{1bc99}\u{1bc9a}\u{1bc9c}\u{1bc9d}\u{1bc9e}\u{1bc9f}\u{1ccef}\u{1ccf0}\u{1ccf9}\u{1ccfa}\u{1ceff}\u{1cf00}\u{1cf2d}\u{1cf2e}\u{1cf2f}\u{1cf30}\u{1cf46}\u{1cf47}\u{1d126}\u{1d129}\u{1d164}\u{1d165}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d241}\u{1d242}\u{1d244}\u{1d245}\u{1d24f}\u{1d253}\u{1d25a}\u{1d25d}\u{1d25e}\u{1d260}\u{1d27f}\u{1d282}\u{1d3ff}\u{1d400}\u{1d454}\u{1d455}\u{1d456}\u{1d49c}\u{1d49d}\u{1d49e}\u{1d49f}\u{1d4a0}\u{1d4a1}\u{1d4a2}\u{1d4a3}\u{1d4a4}\u{1d4a5}\u{1d4a6}\u{1d4a7}\u{1d4a8}\u{1d4a9}\u{1d4ac}\u{1d4ad}\u{1d4ae}\u{1d4b9}\u{1d4ba}\u{1d4bb}\u{1d4bc}\u{1d4bd}\u{1d4c3}\u{1d4c4}\u{1d4c5}\u{1d505}\u{1d506}\u{1d507}\u{1d50a}\u{1d50b}\u{1d50c}\u{1d50d}\u{1d514}\u{1d515}\u{1d516}\u{1d51c}\u{1d51d}\u{1d51e}\u{1d539}\u{1d53a}\u{1d53b}\u{1d53e}\u{1d53f}\u{1d540}\u{1d544}\u{1d545}\u{1d546}\u{1d547}\u{1d549}\u{1d54a}\u{1d550}\u{1d551}\u{1d552}\u{1d6a7}\u{1d6a8}\u{1d6c0}\u{1d6c1}\u{1d6c2}\u{1d6da}\u{1d6db}\u{1d6dc}\u{1d6fa}\u{1d6fb}\u{1d6fc}\u{1d714}\u{1d715}\u{1d716}\u{1d734}\u{1d735}\u{1d736}\u{1d74e}\u{1d74f}\u{1d750}\u{1d76e}\u{1d76f}\u{1d770}\u{1d788}\u{1d789}\u{1d78a}\u{1d7a8}\u{1d7a9}\u{1d7aa}\u{1d7c2}\u{1d7c3}\u{1d7c4}\u{1d7cb}\u{1d7cc}\u{1d7cd}\u{1d7ce}\u{1d7ff}\u{1d800}\u{1d9ff}\u{1da00}\u{1da36}\u{1da37}\u{1da3a}\u{1da3b}\u{1da6c}\u{1da6d}\u{1da74}\u{1da75}\u{1da76}\u{1da83}\u{1da84}\u{1da85}\u{1da9a}\u{1da9b}\u{1da9f}\u{1daa0}\u{1daa1}\u{1daaf}\u{1dab0}\u{1deff}\u{1df00}\u{1df82}\u{1df8f}\u{1df97}\u{1dfcc}\u{1e006}\u{1e007}\u{1e008}\u{1e018}\u{1e019}\u{1e01a}\u{1e01b}\u{1e021}\u{1e022}\u{1e023}\u{1e024}\u{1e025}\u{1e026}\u{1e02a}\u{1e02b}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e08e}\u{1e08f}\u{1e090}\u{1e0ff}\u{1e100}\u{1e12c}\u{1e12d}\u{1e12f}\u{1e130}\u{1e13d}\u{1e13e}\u{1e13f}\u{1e140}\u{1e149}\u{1e14a}\u{1e14d}\u{1e14e}\u{1e14f}\u{1e28f}\u{1e290}\u{1e2ae}\u{1e2af}\u{1e2bf}\u{1e2c0}\u{1e2f9}\u{1e2fa}\u{1e4cf}\u{1e4d0}\u{1e4f9}\u{1e4fa}\u{1e5cf}\u{1e5d0}\u{1e5fa}\u{1e5fb}\u{1e6bf}\u{1e6c0}\u{1e6de}\u{1e6df}\u{1e6e0}\u{1e6f5}\u{1e6f6}\u{1e6fd}\u{1e6fe}\u{1e6ff}\u{1e700}\u{1e7df}\u{1e7e0}\u{1e7e6}\u{1e7e7}\u{1e7e8}\u{1e7eb}\u{1e7ec}\u{1e7ed}\u{1e7ee}\u{1e7ef}\u{1e7f0}\u{1e7fe}\u{1e7ff}\u{1e800}\u{1e8c4}\u{1e8c5}\u{1e8cf}\u{1e8d0}\u{1e8d6}\u{1e8d7}\u{1e8ff}\u{1e900}\u{1e94b}\u{1e94c}\u{1e94f}\u{1e950}\u{1e959}\u{1e95a}\u{1edff}\u{1ee00}\u{1ee03}\u{1ee04}\u{1ee05}\u{1ee1f}\u{1ee20}\u{1ee21}\u{1ee22}\u{1ee23}\u{1ee24}\u{1ee25}\u{1ee26}\u{1ee27}\u{1ee28}\u{1ee29}\u{1ee32}\u{1ee33}\u{1ee34}\u{1ee37}\u{1ee38}\u{1ee39}\u{1ee3a}\u{1ee3b}\u{1ee3c}\u{1ee41}\u{1ee42}\u{1ee43}\u{1ee46}\u{1ee47}\u{1ee48}\u{1ee49}\u{1ee4a}\u{1ee4b}\u{1ee4c}\u{1ee4d}\u{1ee4f}\u{1ee50}\u{1ee51}\u{1ee52}\u{1ee53}\u{1ee54}\u{1ee55}\u{1ee56}\u{1ee57}\u{1ee58}\u{1ee59}\u{1ee5a}\u{1ee5b}\u{1ee5c}\u{1ee5d}\u{1ee5e}\u{1ee5f}\u{1ee60}\u{1ee61}\u{1ee62}\u{1ee63}\u{1ee64}\u{1ee65}\u{1ee66}\u{1ee67}\u{1ee6a}\u{1ee6b}\u{1ee6c}\u{1ee72}\u{1ee73}\u{1ee74}\u{1ee77}\u{1ee78}\u{1ee79}\u{1ee7c}\u{1ee7d}\u{1ee7e}\u{1ee7f}\u{1ee80}\u{1ee89}\u{1ee8a}\u{1ee8b}\u{1ee9b}\u{1ee9c}\u{1eea0}\u{1eea1}\u{1eea3}\u{1eea4}\u{1eea5}\u{1eea9}\u{1eeaa}\u{1eeab}\u{1eebb}\u{1eebc}\u{1fbef}\u{1fbf0}\u{1fbf9}\u{1fbfa}\u{1ffff}\u{20000}\u{2a6df}\u{2a6e0}\u{2a6ff}\u{2a700}\u{2b81f}\u{2b820}\u{2cead}\u{2ceae}\u{2ceaf}\u{2ceb0}\u{2ebe0}\u{2ebe1}\u{2ebef}\u{2ebf0}\u{2ee5d}\u{2ee5e}\u{2f7ff}\u{2f800}\u{2fa1d}\u{2fa1e}\u{2ffff}\u{30000}\u{3134a}\u{3134b}\u{3134f}\u{31350}\u{33479}\u{3347a}\u{3cfff}\u{3fc40}\u{e00ff}\u{e0100}\u{e01ef}\u{e01f0}\u{feff}ſK\n",
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
fn unicode_id_continue_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{IDC}}$/i{mode},q=/^\p{{ID_Continue}}$/i{mode},n=/^\P{{IDC}}$/i{mode};'AſK\u200C\uFEFF'.split('').every(c=>p.test(c)===q.test(c)&&n.test(c)!==p.test(c))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{ID_Continue}}/{mode},b=/\p{{IDC}}/i{mode},n=/\P{{IDC}}/i{mode};a.test('\u0041')&&a.test('\u0042')&&b.test('\u0041')&&!a.test('\u0021')&&b.test('\u005f')&&b.test('ſ')&&b.test('K')&&!n.test('\u0041')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{ID_Continue}}/{mode},s=/\s/{mode};p.test('\u0041')&&p.test('\u0042')&&!s.test('\u0041')&&!s.test('\u0042')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{IDC}}K]/i{mode},b=/[^\P{{ID_Continue}}K]/i{mode},c=/[\p{{IDC}}\P{{ID_Continue}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0041')&&a.test('ſ')&&b.test('\u0042')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{IDC}}$/{mode},b=/^\P{{IDC}}$/{mode};a.test('\u0041')&&!a.test('\u0042\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_id_continue_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{IDC}}K]))$/dim{mode}g,m=r.exec('\u0021\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{IDC}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{IDC}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===2&&m[0]==='\u0041'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{IDC}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0041')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_id_continue_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0042\uDB40\uDC01\u0041\u0021'.matchAll(/(?<x>\P{{IDC}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u0021'"
        ));
        check(&format!(
            r"'\u0042\u0041\uDB40\uDC01\u0021'.replace(/(?<x>\p{{ID_Continue}})/{mode}g,'<$1:$<x>>')==='<\u0042:\u0042><\u0041:\u0041>\uDB40\uDC01\u0021'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u0042\u0041\u0021'.split(/(\p{{IDC}})/{mode}).join('|')==='\uDB40\uDC01|\u0042||\u0041|\u0021'&&'\uDB40\uDC01\u0041'.search(/\p{{IDC}}/{mode})===2&&'\u0042\u0041\uDB40\uDC01'.match(/\p{{IDC}}/{mode}g).join('')==='\u0042\u0041'"
        ));
        check(&format!(
            r"let t='';let x='\u0041\u0042\uDB40\uDC01\u0041'.replace(/(\p{{IDC}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_id_continue_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{ID_Continue})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{ID_Continue})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0041').groups.x==='\u0041'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{IDC}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{IDC}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{IDC}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_id_continue_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{IDC}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u0041'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0041'&&m[3000]==='\\u0041'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_id_continue_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    // Keep plan construction and search-input allocation in separate Scripts.
    // Each Script receives the same explicitly opted-in work budget.
    realm.eval("let r=/\\p{IDC}/ug,marker=0").unwrap();
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
    check(
        r#"let m=/\p{Script_Extensions=Latin}/u.exec('A');JSON.stringify(m)==='["A"]'&&m.index===0&&m.input==='A'"#,
    );
    for source in [
        r"/[\p{IDC}&&A]/v.exec('A')",
        r"/[\p{IDC}--A]/v.exec('A')",
        r"/\p{IDC}+b/u.exec('AA')",
        r"/\p{IDC}A/v.exec('AA')",
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
fn unicode_id_continue_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\b(?<x>\p{{IDC}})+\b/d{mode}g,m=r.exec('\uDB40\uDC01\u0041\u0042\u0042\uFEFF');m.index===2&&m[0]==='\u0041\u0042\u0042'&&m.groups.x==='\u0042'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_id_continue_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x5c8u32, 0x5c9u32, 0xb53u32, 0xb54u32, 0x1adeu32, 0x1adfu32,
        0x1aecu32, 0x1aeeu32, 0x1af0u32, 0x208fu32, 0x209du32, 0x209eu32, 0x209fu32, 0xa7ddu32,
        0xa7e2u32, 0xab6cu32, 0xab6du32, 0x107bbu32, 0x107bdu32, 0x107bfu32, 0x10ecbu32,
        0x10ecdu32, 0x10ecfu32, 0x10ed9u32, 0x10ee3u32, 0x10eeeu32, 0x10ef0u32, 0x10ef4u32,
        0x10ef9u32, 0x11b0au32, 0x11df0u32, 0x11df1u32, 0x1246fu32, 0x12475u32, 0x1247au32,
        0x1247fu32, 0x12550u32, 0x125ebu32, 0x12686u32, 0x18cd6u32, 0x18cd8u32, 0x18cdau32,
        0x18d1fu32, 0x18d20u32, 0x18e00u32, 0x18fc8u32, 0x19191u32, 0x191a0u32, 0x191b9u32,
        0x191d2u32, 0x1b123u32, 0x1b125u32, 0x1b128u32, 0x1b168u32, 0x1d127u32, 0x1d128u32,
        0x1d250u32, 0x1d251u32, 0x1d252u32, 0x1d25bu32, 0x1d25cu32, 0x1d25fu32, 0x1d280u32,
        0x1d281u32, 0x1d6a6u32, 0x1df1fu32, 0x1df21u32, 0x1df24u32, 0x1df2bu32, 0x1df40u32,
        0x1df48u32, 0x1df4au32, 0x1df4du32, 0x1df51u32, 0x1df56u32, 0x1df68u32, 0x1df6au32,
        0x1df6cu32, 0x1df6eu32, 0x1df72u32, 0x1df74u32, 0x1df76u32, 0x1df78u32, 0x1df7au32,
        0x1df7cu32, 0x1df7eu32, 0x1df81u32, 0x1df90u32, 0x1df93u32, 0x1df95u32, 0x1df96u32,
        0x1dfcdu32, 0x1dfe6u32, 0x1dfffu32, 0x2b81eu32, 0x3d000u32, 0x3e61fu32, 0x3fc3fu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["ID_Continue", "IDC"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 808);
}
