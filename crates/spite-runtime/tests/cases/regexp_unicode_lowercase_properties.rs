//! Lowercase and its exact Lower alias over complete Unicode characters.
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
fn unicode_lowercase_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Lowercase}",
        r"\P{Lower}",
        r"[^\p{Lower}]",
        r"[\p{Lower}K]",
        r"[\P{Lowercase}K]",
        r"^(?<x>[\p{Lower}K])$",
        r"(?<x>\P{Lowercase})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{41}\u{42}\u{43}\u{44}\u{45}\u{46}\u{47}\u{48}\u{49}\u{4a}\u{4b}\u{4c}\u{4d}\u{4e}\u{4f}\u{50}\u{51}\u{52}\u{53}\u{54}\u{55}\u{56}\u{57}\u{58}\u{59}\u{5a}\u{60}\u{61}\u{62}\u{63}\u{64}\u{65}\u{66}\u{67}\u{68}\u{69}\u{6a}\u{6b}\u{6c}\u{6d}\u{6e}\u{6f}\u{70}\u{71}\u{72}\u{73}\u{74}\u{75}\u{76}\u{77}\u{78}\u{79}\u{7a}\u{7b}\u{a9}\u{aa}\u{ab}\u{b4}\u{b5}\u{b6}\u{b9}\u{ba}\u{bb}\u{c0}\u{c1}\u{c2}\u{c3}\u{c4}\u{c5}\u{c6}\u{c7}\u{c8}\u{c9}\u{ca}\u{cb}\u{cc}\u{cd}\u{ce}\u{cf}\u{d0}\u{d1}\u{d2}\u{d3}\u{d4}\u{d5}\u{d6}\u{d8}\u{d9}\u{da}\u{db}\u{dc}\u{dd}\u{de}\u{df}\u{e0}\u{e1}\u{e2}\u{e3}\u{e4}\u{e5}\u{e6}\u{e7}\u{e8}\u{e9}\u{ea}\u{eb}\u{ec}\u{ed}\u{ee}\u{ef}\u{f0}\u{f1}\u{f2}\u{f3}\u{f4}\u{f5}\u{f6}\u{f7}\u{f8}\u{f9}\u{fa}\u{fb}\u{fc}\u{fd}\u{fe}\u{ff}\u{100}\u{101}\u{102}\u{103}\u{104}\u{105}\u{106}\u{107}\u{108}\u{109}\u{10a}\u{10b}\u{10c}\u{10d}\u{10e}\u{10f}\u{110}\u{111}\u{112}\u{113}\u{114}\u{115}\u{116}\u{117}\u{118}\u{119}\u{11a}\u{11b}\u{11c}\u{11d}\u{11e}\u{11f}\u{120}\u{121}\u{122}\u{123}\u{124}\u{125}\u{126}\u{127}\u{128}\u{129}\u{12a}\u{12b}\u{12c}\u{12d}\u{12e}\u{12f}\u{130}\u{131}\u{132}\u{133}\u{134}\u{135}\u{136}\u{137}\u{138}\u{139}\u{13a}\u{13b}\u{13c}\u{13d}\u{13e}\u{13f}\u{140}\u{141}\u{142}\u{143}\u{144}\u{145}\u{146}\u{147}\u{148}\u{149}\u{14a}\u{14b}\u{14c}\u{14d}\u{14e}\u{14f}\u{150}\u{151}\u{152}\u{153}\u{154}\u{155}\u{156}\u{157}\u{158}\u{159}\u{15a}\u{15b}\u{15c}\u{15d}\u{15e}\u{15f}\u{160}\u{161}\u{162}\u{163}\u{164}\u{165}\u{166}\u{167}\u{168}\u{169}\u{16a}\u{16b}\u{16c}\u{16d}\u{16e}\u{16f}\u{170}\u{171}\u{172}\u{173}\u{174}\u{175}\u{176}\u{177}\u{178}\u{179}\u{17a}\u{17b}\u{17c}\u{17d}\u{17e}\u{17f}\u{180}\u{181}\u{182}\u{183}\u{184}\u{185}\u{186}\u{187}\u{188}\u{189}\u{18a}\u{18b}\u{18c}\u{18d}\u{18e}\u{18f}\u{190}\u{191}\u{192}\u{193}\u{194}\u{195}\u{196}\u{197}\u{198}\u{199}\u{19a}\u{19b}\u{19c}\u{19d}\u{19e}\u{19f}\u{1a0}\u{1a1}\u{1a2}\u{1a3}\u{1a4}\u{1a5}\u{1a6}\u{1a7}\u{1a8}\u{1a9}\u{1aa}\u{1ab}\u{1ac}\u{1ad}\u{1ae}\u{1af}\u{1b0}\u{1b1}\u{1b2}\u{1b3}\u{1b4}\u{1b5}\u{1b6}\u{1b7}\u{1b8}\u{1b9}\u{1ba}\u{1bb}\u{1bc}\u{1bd}\u{1bf}\u{1c0}\u{1c4}\u{1c5}\u{1c6}\u{1c7}\u{1c8}\u{1c9}\u{1ca}\u{1cb}\u{1cc}\u{1cd}\u{1ce}\u{1cf}\u{1d0}\u{1d1}\u{1d2}\u{1d3}\u{1d4}\u{1d5}\u{1d6}\u{1d7}\u{1d8}\u{1d9}\u{1da}\u{1db}\u{1dc}\u{1dd}\u{1de}\u{1df}\u{1e0}\u{1e1}\u{1e2}\u{1e3}\u{1e4}\u{1e5}\u{1e6}\u{1e7}\u{1e8}\u{1e9}\u{1ea}\u{1eb}\u{1ec}\u{1ed}\u{1ee}\u{1ef}\u{1f0}\u{1f1}\u{1f2}\u{1f3}\u{1f4}\u{1f5}\u{1f6}\u{1f7}\u{1f8}\u{1f9}\u{1fa}\u{1fb}\u{1fc}\u{1fd}\u{1fe}\u{1ff}\u{200}\u{201}\u{202}\u{203}\u{204}\u{205}\u{206}\u{207}\u{208}\u{209}\u{20a}\u{20b}\u{20c}\u{20d}\u{20e}\u{20f}\u{210}\u{211}\u{212}\u{213}\u{214}\u{215}\u{216}\u{217}\u{218}\u{219}\u{21a}\u{21b}\u{21c}\u{21d}\u{21e}\u{21f}\u{220}\u{221}\u{222}\u{223}\u{224}\u{225}\u{226}\u{227}\u{228}\u{229}\u{22a}\u{22b}\u{22c}\u{22d}\u{22e}\u{22f}\u{230}\u{231}\u{232}\u{233}\u{239}\u{23a}\u{23b}\u{23c}\u{23d}\u{23e}\u{23f}\u{240}\u{241}\u{242}\u{243}\u{244}\u{245}\u{246}\u{247}\u{248}\u{249}\u{24a}\u{24b}\u{24c}\u{24d}\u{24e}\u{24f}\u{250}\u{251}\u{252}\u{253}\u{254}\u{256}\u{257}\u{259}\u{25b}\u{25c}\u{260}\u{261}\u{263}\u{264}\u{265}\u{266}\u{268}\u{269}\u{26a}\u{26b}\u{26c}\u{26f}\u{271}\u{272}\u{275}\u{27d}\u{280}\u{282}\u{283}\u{287}\u{288}\u{289}\u{28a}\u{28b}\u{28c}\u{292}\u{293}\u{294}\u{295}\u{296}\u{29d}\u{29e}\u{2b8}\u{2b9}\u{2bf}\u{2c0}\u{2c1}\u{2c2}\u{2df}\u{2e0}\u{2e4}\u{2e5}\u{344}\u{345}\u{346}\u{370}\u{371}\u{372}\u{373}\u{374}\u{376}\u{377}\u{378}\u{379}\u{37a}\u{37b}\u{37c}\u{37d}\u{37e}\u{37f}\u{386}\u{388}\u{389}\u{38a}\u{38c}\u{38e}\u{38f}\u{390}\u{391}\u{392}\u{393}\u{394}\u{395}\u{396}\u{397}\u{398}\u{399}\u{39a}\u{39b}\u{39c}\u{39d}\u{39e}\u{39f}\u{3a0}\u{3a1}\u{3a3}\u{3a4}\u{3a5}\u{3a6}\u{3a7}\u{3a8}\u{3a9}\u{3aa}\u{3ab}\u{3ac}\u{3ad}\u{3ae}\u{3af}\u{3b1}\u{3b2}\u{3b3}\u{3b4}\u{3b5}\u{3b6}\u{3b7}\u{3b8}\u{3b9}\u{3ba}\u{3bb}\u{3bc}\u{3bd}\u{3be}\u{3bf}\u{3c0}\u{3c1}\u{3c2}\u{3c3}\u{3c4}\u{3c5}\u{3c6}\u{3c7}\u{3c8}\u{3c9}\u{3ca}\u{3cb}\u{3cc}\u{3cd}\u{3ce}\u{3cf}\u{3d0}\u{3d1}\u{3d2}\u{3d4}\u{3d5}\u{3d6}\u{3d7}\u{3d8}\u{3d9}\u{3da}\u{3db}\u{3dc}\u{3dd}\u{3de}\u{3df}\u{3e0}\u{3e1}\u{3e2}\u{3e3}\u{3e4}\u{3e5}\u{3e6}\u{3e7}\u{3e8}\u{3e9}\u{3ea}\u{3eb}\u{3ec}\u{3ed}\u{3ee}\u{3ef}\u{3f0}\u{3f1}\u{3f2}\u{3f3}\u{3f4}\u{3f5}\u{3f6}\u{3f7}\u{3f8}\u{3f9}\u{3fa}\u{3fb}\u{3fc}\u{3fd}\u{3fe}\u{3ff}\u{400}\u{401}\u{402}\u{403}\u{404}\u{405}\u{406}\u{407}\u{408}\u{409}\u{40a}\u{40b}\u{40c}\u{40d}\u{40e}\u{40f}\u{410}\u{411}\u{412}\u{413}\u{414}\u{415}\u{416}\u{417}\u{418}\u{419}\u{41a}\u{41b}\u{41c}\u{41d}\u{41e}\u{41f}\u{420}\u{421}\u{422}\u{423}\u{424}\u{425}\u{426}\u{427}\u{428}\u{429}\u{42a}\u{42b}\u{42c}\u{42d}\u{42e}\u{42f}\u{430}\u{431}\u{432}\u{433}\u{434}\u{435}\u{436}\u{437}\u{438}\u{439}\u{43a}\u{43b}\u{43c}\u{43d}\u{43e}\u{43f}\u{440}\u{441}\u{442}\u{443}\u{444}\u{445}\u{446}\u{447}\u{448}\u{449}\u{44a}\u{44b}\u{44c}\u{44d}\u{44e}\u{44f}\u{450}\u{451}\u{452}\u{453}\u{454}\u{455}\u{456}\u{457}\u{458}\u{459}\u{45a}\u{45b}\u{45c}\u{45d}\u{45e}\u{45f}\u{460}\u{461}\u{462}\u{463}\u{464}\u{465}\u{466}\u{467}\u{468}\u{469}\u{46a}\u{46b}\u{46c}\u{46d}\u{46e}\u{46f}\u{470}\u{471}\u{472}\u{473}\u{474}\u{475}\u{476}\u{477}\u{478}\u{479}\u{47a}\u{47b}\u{47c}\u{47d}\u{47e}\u{47f}\u{480}\u{481}\u{482}\u{48a}\u{48b}\u{48c}\u{48d}\u{48e}\u{48f}\u{490}\u{491}\u{492}\u{493}\u{494}\u{495}\u{496}\u{497}\u{498}\u{499}\u{49a}\u{49b}\u{49c}\u{49d}\u{49e}\u{49f}\u{4a0}\u{4a1}\u{4a2}\u{4a3}\u{4a4}\u{4a5}\u{4a6}\u{4a7}\u{4a8}\u{4a9}\u{4aa}\u{4ab}\u{4ac}\u{4ad}\u{4ae}\u{4af}\u{4b0}\u{4b1}\u{4b2}\u{4b3}\u{4b4}\u{4b5}\u{4b6}\u{4b7}\u{4b8}\u{4b9}\u{4ba}\u{4bb}\u{4bc}\u{4bd}\u{4be}\u{4bf}\u{4c0}\u{4c1}\u{4c2}\u{4c3}\u{4c4}\u{4c5}\u{4c6}\u{4c7}\u{4c8}\u{4c9}\u{4ca}\u{4cb}\u{4cc}\u{4cd}\u{4ce}\u{4cf}\u{4d0}\u{4d1}\u{4d2}\u{4d3}\u{4d4}\u{4d5}\u{4d6}\u{4d7}\u{4d8}\u{4d9}\u{4da}\u{4db}\u{4dc}\u{4dd}\u{4de}\u{4df}\u{4e0}\u{4e1}\u{4e2}\u{4e3}\u{4e4}\u{4e5}\u{4e6}\u{4e7}\u{4e8}\u{4e9}\u{4ea}\u{4eb}\u{4ec}\u{4ed}\u{4ee}\u{4ef}\u{4f0}\u{4f1}\u{4f2}\u{4f3}\u{4f4}\u{4f5}\u{4f6}\u{4f7}\u{4f8}\u{4f9}\u{4fa}\u{4fb}\u{4fc}\u{4fd}\u{4fe}\u{4ff}\u{500}\u{501}\u{502}\u{503}\u{504}\u{505}\u{506}\u{507}\u{508}\u{509}\u{50a}\u{50b}\u{50c}\u{50d}\u{50e}\u{50f}\u{510}\u{511}\u{512}\u{513}\u{514}\u{515}\u{516}\u{517}\u{518}\u{519}\u{51a}\u{51b}\u{51c}\u{51d}\u{51e}\u{51f}\u{520}\u{521}\u{522}\u{523}\u{524}\u{525}\u{526}\u{527}\u{528}\u{529}\u{52a}\u{52b}\u{52c}\u{52d}\u{52e}\u{52f}\u{530}\u{531}\u{532}\u{533}\u{534}\u{535}\u{536}\u{537}\u{538}\u{539}\u{53a}\u{53b}\u{53c}\u{53d}\u{53e}\u{53f}\u{540}\u{541}\u{542}\u{543}\u{544}\u{545}\u{546}\u{547}\u{548}\u{549}\u{54a}\u{54b}\u{54c}\u{54d}\u{54e}\u{54f}\u{550}\u{551}\u{552}\u{553}\u{554}\u{555}\u{556}\u{557}\u{559}\u{55f}\u{560}\u{561}\u{562}\u{563}\u{564}\u{565}\u{566}\u{567}\u{568}\u{569}\u{56a}\u{56b}\u{56c}\u{56d}\u{56e}\u{56f}\u{570}\u{571}\u{572}\u{573}\u{574}\u{575}\u{576}\u{577}\u{578}\u{579}\u{57a}\u{57b}\u{57c}\u{57d}\u{57e}\u{57f}\u{580}\u{581}\u{582}\u{583}\u{584}\u{585}\u{586}\u{588}\u{589}\u{58a}\u{58d}\u{10a0}\u{10a1}\u{10a2}\u{10a3}\u{10a4}\u{10a5}\u{10a6}\u{10a7}\u{10a8}\u{10a9}\u{10aa}\u{10ab}\u{10ac}\u{10ad}\u{10ae}\u{10af}\u{10b0}\u{10b1}\u{10b2}\u{10b3}\u{10b4}\u{10b5}\u{10b6}\u{10b7}\u{10b8}\u{10b9}\u{10ba}\u{10bb}\u{10bc}\u{10bd}\u{10be}\u{10bf}\u{10c0}\u{10c1}\u{10c2}\u{10c3}\u{10c4}\u{10c5}\u{10c7}\u{10cd}\u{10cf}\u{10d0}\u{10d1}\u{10d2}\u{10d3}\u{10d4}\u{10d5}\u{10d6}\u{10d7}\u{10d8}\u{10d9}\u{10da}\u{10db}\u{10dc}\u{10dd}\u{10de}\u{10df}\u{10e0}\u{10e1}\u{10e2}\u{10e3}\u{10e4}\u{10e5}\u{10e6}\u{10e7}\u{10e8}\u{10e9}\u{10ea}\u{10eb}\u{10ec}\u{10ed}\u{10ee}\u{10ef}\u{10f0}\u{10f1}\u{10f2}\u{10f3}\u{10f4}\u{10f5}\u{10f6}\u{10f7}\u{10f8}\u{10f9}\u{10fa}\u{10fb}\u{10fc}\u{10fd}\u{10fe}\u{10ff}\u{1100}\u{13a0}\u{13a1}\u{13a2}\u{13a3}\u{13a4}\u{13a5}\u{13a6}\u{13a7}\u{13a8}\u{13a9}\u{13aa}\u{13ab}\u{13ac}\u{13ad}\u{13ae}\u{13af}\u{13b0}\u{13b1}\u{13b2}\u{13b3}\u{13b4}\u{13b5}\u{13b6}\u{13b7}\u{13b8}\u{13b9}\u{13ba}\u{13bb}\u{13bc}\u{13bd}\u{13be}\u{13bf}\u{13c0}\u{13c1}\u{13c2}\u{13c3}\u{13c4}\u{13c5}\u{13c6}\u{13c7}\u{13c8}\u{13c9}\u{13ca}\u{13cb}\u{13cc}\u{13cd}\u{13ce}\u{13cf}\u{13d0}\u{13d1}\u{13d2}\u{13d3}\u{13d4}\u{13d5}\u{13d6}\u{13d7}\u{13d8}\u{13d9}\u{13da}\u{13db}\u{13dc}\u{13dd}\u{13de}\u{13df}\u{13e0}\u{13e1}\u{13e2}\u{13e3}\u{13e4}\u{13e5}\u{13e6}\u{13e7}\u{13e8}\u{13e9}\u{13ea}\u{13eb}\u{13ec}\u{13ed}\u{13ee}\u{13ef}\u{13f0}\u{13f1}\u{13f2}\u{13f3}\u{13f4}\u{13f5}\u{13f7}\u{13f8}\u{13f9}\u{13fa}\u{13fb}\u{13fc}\u{13fd}\u{13fe}\u{1c7f}\u{1c80}\u{1c81}\u{1c82}\u{1c83}\u{1c84}\u{1c85}\u{1c86}\u{1c87}\u{1c88}\u{1c89}\u{1c8a}\u{1c8b}\u{1c90}\u{1c91}\u{1c92}\u{1c93}\u{1c94}\u{1c95}\u{1c96}\u{1c97}\u{1c98}\u{1c99}\u{1c9a}\u{1c9b}\u{1c9c}\u{1c9d}\u{1c9e}\u{1c9f}\u{1ca0}\u{1ca1}\u{1ca2}\u{1ca3}\u{1ca4}\u{1ca5}\u{1ca6}\u{1ca7}\u{1ca8}\u{1ca9}\u{1caa}\u{1cab}\u{1cac}\u{1cad}\u{1cae}\u{1caf}\u{1cb0}\u{1cb1}\u{1cb2}\u{1cb3}\u{1cb4}\u{1cb5}\u{1cb6}\u{1cb7}\u{1cb8}\u{1cb9}\u{1cba}\u{1cbd}\u{1cbe}\u{1cbf}\u{1cff}\u{1d00}\u{1d79}\u{1d7d}\u{1d8e}\u{1dbf}\u{1dc0}\u{1e00}\u{1e01}\u{1e02}\u{1e03}\u{1e04}\u{1e05}\u{1e06}\u{1e07}\u{1e08}\u{1e09}\u{1e0a}\u{1e0b}\u{1e0c}\u{1e0d}\u{1e0e}\u{1e0f}\u{1e10}\u{1e11}\u{1e12}\u{1e13}\u{1e14}\u{1e15}\u{1e16}\u{1e17}\u{1e18}\u{1e19}\u{1e1a}\u{1e1b}\u{1e1c}\u{1e1d}\u{1e1e}\u{1e1f}\u{1e20}\u{1e21}\u{1e22}\u{1e23}\u{1e24}\u{1e25}\u{1e26}\u{1e27}\u{1e28}\u{1e29}\u{1e2a}\u{1e2b}\u{1e2c}\u{1e2d}\u{1e2e}\u{1e2f}\u{1e30}\u{1e31}\u{1e32}\u{1e33}\u{1e34}\u{1e35}\u{1e36}\u{1e37}\u{1e38}\u{1e39}\u{1e3a}\u{1e3b}\u{1e3c}\u{1e3d}\u{1e3e}\u{1e3f}\u{1e40}\u{1e41}\u{1e42}\u{1e43}\u{1e44}\u{1e45}\u{1e46}\u{1e47}\u{1e48}\u{1e49}\u{1e4a}\u{1e4b}\u{1e4c}\u{1e4d}\u{1e4e}\u{1e4f}\u{1e50}\u{1e51}\u{1e52}\u{1e53}\u{1e54}\u{1e55}\u{1e56}\u{1e57}\u{1e58}\u{1e59}\u{1e5a}\u{1e5b}\u{1e5c}\u{1e5d}\u{1e5e}\u{1e5f}\u{1e60}\u{1e61}\u{1e62}\u{1e63}\u{1e64}\u{1e65}\u{1e66}\u{1e67}\u{1e68}\u{1e69}\u{1e6a}\u{1e6b}\u{1e6c}\u{1e6d}\u{1e6e}\u{1e6f}\u{1e70}\u{1e71}\u{1e72}\u{1e73}\u{1e74}\u{1e75}\u{1e76}\u{1e77}\u{1e78}\u{1e79}\u{1e7a}\u{1e7b}\u{1e7c}\u{1e7d}\u{1e7e}\u{1e7f}\u{1e80}\u{1e81}\u{1e82}\u{1e83}\u{1e84}\u{1e85}\u{1e86}\u{1e87}\u{1e88}\u{1e89}\u{1e8a}\u{1e8b}\u{1e8c}\u{1e8d}\u{1e8e}\u{1e8f}\u{1e90}\u{1e91}\u{1e92}\u{1e93}\u{1e94}\u{1e95}\u{1e9b}\u{1e9d}\u{1e9e}\u{1e9f}\u{1ea0}\u{1ea1}\u{1ea2}\u{1ea3}\u{1ea4}\u{1ea5}\u{1ea6}\u{1ea7}\u{1ea8}\u{1ea9}\u{1eaa}\u{1eab}\u{1eac}\u{1ead}\u{1eae}\u{1eaf}\u{1eb0}\u{1eb1}\u{1eb2}\u{1eb3}\u{1eb4}\u{1eb5}\u{1eb6}\u{1eb7}\u{1eb8}\u{1eb9}\u{1eba}\u{1ebb}\u{1ebc}\u{1ebd}\u{1ebe}\u{1ebf}\u{1ec0}\u{1ec1}\u{1ec2}\u{1ec3}\u{1ec4}\u{1ec5}\u{1ec6}\u{1ec7}\u{1ec8}\u{1ec9}\u{1eca}\u{1ecb}\u{1ecc}\u{1ecd}\u{1ece}\u{1ecf}\u{1ed0}\u{1ed1}\u{1ed2}\u{1ed3}\u{1ed4}\u{1ed5}\u{1ed6}\u{1ed7}\u{1ed8}\u{1ed9}\u{1eda}\u{1edb}\u{1edc}\u{1edd}\u{1ede}\u{1edf}\u{1ee0}\u{1ee1}\u{1ee2}\u{1ee3}\u{1ee4}\u{1ee5}\u{1ee6}\u{1ee7}\u{1ee8}\u{1ee9}\u{1eea}\u{1eeb}\u{1eec}\u{1eed}\u{1eee}\u{1eef}\u{1ef0}\u{1ef1}\u{1ef2}\u{1ef3}\u{1ef4}\u{1ef5}\u{1ef6}\u{1ef7}\u{1ef8}\u{1ef9}\u{1efa}\u{1efb}\u{1efc}\u{1efd}\u{1efe}\u{1eff}\u{1f00}\u{1f01}\u{1f02}\u{1f03}\u{1f04}\u{1f05}\u{1f06}\u{1f07}\u{1f08}\u{1f09}\u{1f0a}\u{1f0b}\u{1f0c}\u{1f0d}\u{1f0e}\u{1f0f}\u{1f10}\u{1f11}\u{1f12}\u{1f13}\u{1f14}\u{1f15}\u{1f16}\u{1f18}\u{1f19}\u{1f1a}\u{1f1b}\u{1f1c}\u{1f1d}\u{1f1f}\u{1f20}\u{1f21}\u{1f22}\u{1f23}\u{1f24}\u{1f25}\u{1f26}\u{1f27}\u{1f28}\u{1f29}\u{1f2a}\u{1f2b}\u{1f2c}\u{1f2d}\u{1f2e}\u{1f2f}\u{1f30}\u{1f31}\u{1f32}\u{1f33}\u{1f34}\u{1f35}\u{1f36}\u{1f37}\u{1f38}\u{1f39}\u{1f3a}\u{1f3b}\u{1f3c}\u{1f3d}\u{1f3e}\u{1f3f}\u{1f40}\u{1f41}\u{1f42}\u{1f43}\u{1f44}\u{1f45}\u{1f46}\u{1f48}\u{1f49}\u{1f4a}\u{1f4b}\u{1f4c}\u{1f4d}\u{1f4f}\u{1f50}\u{1f51}\u{1f53}\u{1f55}\u{1f57}\u{1f58}\u{1f59}\u{1f5b}\u{1f5d}\u{1f5f}\u{1f60}\u{1f61}\u{1f62}\u{1f63}\u{1f64}\u{1f65}\u{1f66}\u{1f67}\u{1f68}\u{1f69}\u{1f6a}\u{1f6b}\u{1f6c}\u{1f6d}\u{1f6e}\u{1f6f}\u{1f70}\u{1f71}\u{1f72}\u{1f73}\u{1f74}\u{1f75}\u{1f76}\u{1f77}\u{1f78}\u{1f79}\u{1f7a}\u{1f7b}\u{1f7c}\u{1f7d}\u{1f7e}\u{1f7f}\u{1f80}\u{1f81}\u{1f82}\u{1f83}\u{1f84}\u{1f85}\u{1f86}\u{1f87}\u{1f88}\u{1f89}\u{1f8a}\u{1f8b}\u{1f8c}\u{1f8d}\u{1f8e}\u{1f8f}\u{1f90}\u{1f91}\u{1f92}\u{1f93}\u{1f94}\u{1f95}\u{1f96}\u{1f97}\u{1f98}\u{1f99}\u{1f9a}\u{1f9b}\u{1f9c}\u{1f9d}\u{1f9e}\u{1f9f}\u{1fa0}\u{1fa1}\u{1fa2}\u{1fa3}\u{1fa4}\u{1fa5}\u{1fa6}\u{1fa7}\u{1fa8}\u{1fa9}\u{1faa}\u{1fab}\u{1fac}\u{1fad}\u{1fae}\u{1faf}\u{1fb0}\u{1fb1}\u{1fb3}\u{1fb4}\u{1fb5}\u{1fb6}\u{1fb7}\u{1fb8}\u{1fb9}\u{1fba}\u{1fbb}\u{1fbc}\u{1fbd}\u{1fbe}\u{1fbf}\u{1fc1}\u{1fc2}\u{1fc3}\u{1fc4}\u{1fc5}\u{1fc6}\u{1fc7}\u{1fc8}\u{1fc9}\u{1fca}\u{1fcb}\u{1fcc}\u{1fcf}\u{1fd0}\u{1fd1}\u{1fd3}\u{1fd4}\u{1fd5}\u{1fd6}\u{1fd7}\u{1fd8}\u{1fd9}\u{1fda}\u{1fdb}\u{1fdf}\u{1fe0}\u{1fe1}\u{1fe5}\u{1fe7}\u{1fe8}\u{1fe9}\u{1fea}\u{1feb}\u{1fec}\u{1ff1}\u{1ff2}\u{1ff3}\u{1ff4}\u{1ff5}\u{1ff6}\u{1ff7}\u{1ff8}\u{1ff9}\u{1ffa}\u{1ffb}\u{1ffc}\u{2070}\u{2071}\u{2072}\u{207e}\u{207f}\u{2080}\u{208f}\u{2090}\u{20a0}\u{2109}\u{210a}\u{210b}\u{210d}\u{210e}\u{210f}\u{2110}\u{2112}\u{2113}\u{2114}\u{2126}\u{212a}\u{212b}\u{212e}\u{212f}\u{2130}\u{2132}\u{2133}\u{2134}\u{2135}\u{2138}\u{2139}\u{213a}\u{213b}\u{213c}\u{213d}\u{213e}\u{2145}\u{2146}\u{2149}\u{214a}\u{214d}\u{214e}\u{214f}\u{2160}\u{2161}\u{2162}\u{2163}\u{2164}\u{2165}\u{2166}\u{2167}\u{2168}\u{2169}\u{216a}\u{216b}\u{216c}\u{216d}\u{216e}\u{216f}\u{2170}\u{2171}\u{2172}\u{2173}\u{2174}\u{2175}\u{2176}\u{2177}\u{2178}\u{2179}\u{217a}\u{217b}\u{217c}\u{217d}\u{217e}\u{217f}\u{2180}\u{2183}\u{2184}\u{2185}\u{24b6}\u{24b7}\u{24b8}\u{24b9}\u{24ba}\u{24bb}\u{24bc}\u{24bd}\u{24be}\u{24bf}\u{24c0}\u{24c1}\u{24c2}\u{24c3}\u{24c4}\u{24c5}\u{24c6}\u{24c7}\u{24c8}\u{24c9}\u{24ca}\u{24cb}\u{24cc}\u{24cd}\u{24ce}\u{24cf}\u{24d0}\u{24d1}\u{24d2}\u{24d3}\u{24d4}\u{24d5}\u{24d6}\u{24d7}\u{24d8}\u{24d9}\u{24da}\u{24db}\u{24dc}\u{24dd}\u{24de}\u{24df}\u{24e0}\u{24e1}\u{24e2}\u{24e3}\u{24e4}\u{24e5}\u{24e6}\u{24e7}\u{24e8}\u{24e9}\u{24ea}\u{2c00}\u{2c01}\u{2c02}\u{2c03}\u{2c04}\u{2c05}\u{2c06}\u{2c07}\u{2c08}\u{2c09}\u{2c0a}\u{2c0b}\u{2c0c}\u{2c0d}\u{2c0e}\u{2c0f}\u{2c10}\u{2c11}\u{2c12}\u{2c13}\u{2c14}\u{2c15}\u{2c16}\u{2c17}\u{2c18}\u{2c19}\u{2c1a}\u{2c1b}\u{2c1c}\u{2c1d}\u{2c1e}\u{2c1f}\u{2c20}\u{2c21}\u{2c22}\u{2c23}\u{2c24}\u{2c25}\u{2c26}\u{2c27}\u{2c28}\u{2c29}\u{2c2a}\u{2c2b}\u{2c2c}\u{2c2d}\u{2c2e}\u{2c2f}\u{2c30}\u{2c31}\u{2c32}\u{2c33}\u{2c34}\u{2c35}\u{2c36}\u{2c37}\u{2c38}\u{2c39}\u{2c3a}\u{2c3b}\u{2c3c}\u{2c3d}\u{2c3e}\u{2c3f}\u{2c40}\u{2c41}\u{2c42}\u{2c43}\u{2c44}\u{2c45}\u{2c46}\u{2c47}\u{2c48}\u{2c49}\u{2c4a}\u{2c4b}\u{2c4c}\u{2c4d}\u{2c4e}\u{2c4f}\u{2c50}\u{2c51}\u{2c52}\u{2c53}\u{2c54}\u{2c55}\u{2c56}\u{2c57}\u{2c58}\u{2c59}\u{2c5a}\u{2c5b}\u{2c5c}\u{2c5d}\u{2c5e}\u{2c5f}\u{2c60}\u{2c61}\u{2c62}\u{2c63}\u{2c64}\u{2c65}\u{2c66}\u{2c67}\u{2c68}\u{2c69}\u{2c6a}\u{2c6b}\u{2c6c}\u{2c6d}\u{2c6e}\u{2c6f}\u{2c70}\u{2c71}\u{2c72}\u{2c73}\u{2c74}\u{2c75}\u{2c76}\u{2c7d}\u{2c7e}\u{2c7f}\u{2c80}\u{2c81}\u{2c82}\u{2c83}\u{2c84}\u{2c85}\u{2c86}\u{2c87}\u{2c88}\u{2c89}\u{2c8a}\u{2c8b}\u{2c8c}\u{2c8d}\u{2c8e}\u{2c8f}\u{2c90}\u{2c91}\u{2c92}\u{2c93}\u{2c94}\u{2c95}\u{2c96}\u{2c97}\u{2c98}\u{2c99}\u{2c9a}\u{2c9b}\u{2c9c}\u{2c9d}\u{2c9e}\u{2c9f}\u{2ca0}\u{2ca1}\u{2ca2}\u{2ca3}\u{2ca4}\u{2ca5}\u{2ca6}\u{2ca7}\u{2ca8}\u{2ca9}\u{2caa}\u{2cab}\u{2cac}\u{2cad}\u{2cae}\u{2caf}\u{2cb0}\u{2cb1}\u{2cb2}\u{2cb3}\u{2cb4}\u{2cb5}\u{2cb6}\u{2cb7}\u{2cb8}\u{2cb9}\u{2cba}\u{2cbb}\u{2cbc}\u{2cbd}\u{2cbe}\u{2cbf}\u{2cc0}\u{2cc1}\u{2cc2}\u{2cc3}\u{2cc4}\u{2cc5}\u{2cc6}\u{2cc7}\u{2cc8}\u{2cc9}\u{2cca}\u{2ccb}\u{2ccc}\u{2ccd}\u{2cce}\u{2ccf}\u{2cd0}\u{2cd1}\u{2cd2}\u{2cd3}\u{2cd4}\u{2cd5}\u{2cd6}\u{2cd7}\u{2cd8}\u{2cd9}\u{2cda}\u{2cdb}\u{2cdc}\u{2cdd}\u{2cde}\u{2cdf}\u{2ce0}\u{2ce1}\u{2ce2}\u{2ce3}\u{2ce4}\u{2ce5}\u{2ceb}\u{2cec}\u{2ced}\u{2cee}\u{2cef}\u{2cf2}\u{2cf3}\u{2cf4}\u{2cff}\u{2d00}\u{2d01}\u{2d02}\u{2d03}\u{2d04}\u{2d05}\u{2d06}\u{2d07}\u{2d08}\u{2d09}\u{2d0a}\u{2d0b}\u{2d0c}\u{2d0d}\u{2d0e}\u{2d0f}\u{2d10}\u{2d11}\u{2d12}\u{2d13}\u{2d14}\u{2d15}\u{2d16}\u{2d17}\u{2d18}\u{2d19}\u{2d1a}\u{2d1b}\u{2d1c}\u{2d1d}\u{2d1e}\u{2d1f}\u{2d20}\u{2d21}\u{2d22}\u{2d23}\u{2d24}\u{2d25}\u{2d26}\u{2d27}\u{2d28}\u{2d2c}\u{2d2d}\u{2d2e}\u{a640}\u{a641}\u{a642}\u{a643}\u{a644}\u{a645}\u{a646}\u{a647}\u{a648}\u{a649}\u{a64a}\u{a64b}\u{a64c}\u{a64d}\u{a64e}\u{a64f}\u{a650}\u{a651}\u{a652}\u{a653}\u{a654}\u{a655}\u{a656}\u{a657}\u{a658}\u{a659}\u{a65a}\u{a65b}\u{a65c}\u{a65d}\u{a65e}\u{a65f}\u{a660}\u{a661}\u{a662}\u{a663}\u{a664}\u{a665}\u{a666}\u{a667}\u{a668}\u{a669}\u{a66a}\u{a66b}\u{a66c}\u{a66d}\u{a66e}\u{a680}\u{a681}\u{a682}\u{a683}\u{a684}\u{a685}\u{a686}\u{a687}\u{a688}\u{a689}\u{a68a}\u{a68b}\u{a68c}\u{a68d}\u{a68e}\u{a68f}\u{a690}\u{a691}\u{a692}\u{a693}\u{a694}\u{a695}\u{a696}\u{a697}\u{a698}\u{a699}\u{a69a}\u{a69b}\u{a69d}\u{a69e}\u{a722}\u{a723}\u{a724}\u{a725}\u{a726}\u{a727}\u{a728}\u{a729}\u{a72a}\u{a72b}\u{a72c}\u{a72d}\u{a72e}\u{a72f}\u{a731}\u{a732}\u{a733}\u{a734}\u{a735}\u{a736}\u{a737}\u{a738}\u{a739}\u{a73a}\u{a73b}\u{a73c}\u{a73d}\u{a73e}\u{a73f}\u{a740}\u{a741}\u{a742}\u{a743}\u{a744}\u{a745}\u{a746}\u{a747}\u{a748}\u{a749}\u{a74a}\u{a74b}\u{a74c}\u{a74d}\u{a74e}\u{a74f}\u{a750}\u{a751}\u{a752}\u{a753}\u{a754}\u{a755}\u{a756}\u{a757}\u{a758}\u{a759}\u{a75a}\u{a75b}\u{a75c}\u{a75d}\u{a75e}\u{a75f}\u{a760}\u{a761}\u{a762}\u{a763}\u{a764}\u{a765}\u{a766}\u{a767}\u{a768}\u{a769}\u{a76a}\u{a76b}\u{a76c}\u{a76d}\u{a76e}\u{a76f}\u{a778}\u{a779}\u{a77a}\u{a77b}\u{a77c}\u{a77d}\u{a77e}\u{a77f}\u{a780}\u{a781}\u{a782}\u{a783}\u{a784}\u{a785}\u{a786}\u{a787}\u{a788}\u{a78b}\u{a78c}\u{a78d}\u{a78e}\u{a78f}\u{a790}\u{a791}\u{a792}\u{a793}\u{a794}\u{a795}\u{a796}\u{a797}\u{a798}\u{a799}\u{a79a}\u{a79b}\u{a79c}\u{a79d}\u{a79e}\u{a79f}\u{a7a0}\u{a7a1}\u{a7a2}\u{a7a3}\u{a7a4}\u{a7a5}\u{a7a6}\u{a7a7}\u{a7a8}\u{a7a9}\u{a7aa}\u{a7ab}\u{a7ac}\u{a7ad}\u{a7ae}\u{a7af}\u{a7b0}\u{a7b1}\u{a7b2}\u{a7b3}\u{a7b4}\u{a7b5}\u{a7b6}\u{a7b7}\u{a7b8}\u{a7b9}\u{a7ba}\u{a7bb}\u{a7bc}\u{a7bd}\u{a7be}\u{a7bf}\u{a7c0}\u{a7c1}\u{a7c2}\u{a7c3}\u{a7c4}\u{a7c5}\u{a7c6}\u{a7c7}\u{a7c8}\u{a7c9}\u{a7ca}\u{a7cb}\u{a7cc}\u{a7cd}\u{a7ce}\u{a7cf}\u{a7d0}\u{a7d1}\u{a7d2}\u{a7d3}\u{a7d4}\u{a7d5}\u{a7d6}\u{a7d7}\u{a7d8}\u{a7d9}\u{a7da}\u{a7db}\u{a7dc}\u{a7f0}\u{a7f1}\u{a7f4}\u{a7f5}\u{a7f6}\u{a7f7}\u{a7f8}\u{a7fa}\u{a7fb}\u{ab2f}\u{ab30}\u{ab53}\u{ab5a}\u{ab5b}\u{ab5c}\u{ab69}\u{ab6a}\u{ab6f}\u{ab70}\u{ab71}\u{ab72}\u{ab73}\u{ab74}\u{ab75}\u{ab76}\u{ab77}\u{ab78}\u{ab79}\u{ab7a}\u{ab7b}\u{ab7c}\u{ab7d}\u{ab7e}\u{ab7f}\u{ab80}\u{ab81}\u{ab82}\u{ab83}\u{ab84}\u{ab85}\u{ab86}\u{ab87}\u{ab88}\u{ab89}\u{ab8a}\u{ab8b}\u{ab8c}\u{ab8d}\u{ab8e}\u{ab8f}\u{ab90}\u{ab91}\u{ab92}\u{ab93}\u{ab94}\u{ab95}\u{ab96}\u{ab97}\u{ab98}\u{ab99}\u{ab9a}\u{ab9b}\u{ab9c}\u{ab9d}\u{ab9e}\u{ab9f}\u{aba0}\u{aba1}\u{aba2}\u{aba3}\u{aba4}\u{aba5}\u{aba6}\u{aba7}\u{aba8}\u{aba9}\u{abaa}\u{abab}\u{abac}\u{abad}\u{abae}\u{abaf}\u{abb0}\u{abb1}\u{abb2}\u{abb3}\u{abb4}\u{abb5}\u{abb6}\u{abb7}\u{abb8}\u{abb9}\u{abba}\u{abbb}\u{abbc}\u{abbd}\u{abbe}\u{abbf}\u{abc0}\u{faff}\u{fb00}\u{fb06}\u{fb07}\u{fb12}\u{fb13}\u{fb17}\u{fb18}\u{ff21}\u{ff22}\u{ff23}\u{ff24}\u{ff25}\u{ff26}\u{ff27}\u{ff28}\u{ff29}\u{ff2a}\u{ff2b}\u{ff2c}\u{ff2d}\u{ff2e}\u{ff2f}\u{ff30}\u{ff31}\u{ff32}\u{ff33}\u{ff34}\u{ff35}\u{ff36}\u{ff37}\u{ff38}\u{ff39}\u{ff3a}\u{ff40}\u{ff41}\u{ff42}\u{ff43}\u{ff44}\u{ff45}\u{ff46}\u{ff47}\u{ff48}\u{ff49}\u{ff4a}\u{ff4b}\u{ff4c}\u{ff4d}\u{ff4e}\u{ff4f}\u{ff50}\u{ff51}\u{ff52}\u{ff53}\u{ff54}\u{ff55}\u{ff56}\u{ff57}\u{ff58}\u{ff59}\u{ff5a}\u{ff5b}\u{10400}\u{10401}\u{10402}\u{10403}\u{10404}\u{10405}\u{10406}\u{10407}\u{10408}\u{10409}\u{1040a}\u{1040b}\u{1040c}\u{1040d}\u{1040e}\u{1040f}\u{10410}\u{10411}\u{10412}\u{10413}\u{10414}\u{10415}\u{10416}\u{10417}\u{10418}\u{10419}\u{1041a}\u{1041b}\u{1041c}\u{1041d}\u{1041e}\u{1041f}\u{10420}\u{10421}\u{10422}\u{10423}\u{10424}\u{10425}\u{10426}\u{10427}\u{10428}\u{10429}\u{1042a}\u{1042b}\u{1042c}\u{1042d}\u{1042e}\u{1042f}\u{10430}\u{10431}\u{10432}\u{10433}\u{10434}\u{10435}\u{10436}\u{10437}\u{10438}\u{10439}\u{1043a}\u{1043b}\u{1043c}\u{1043d}\u{1043e}\u{1043f}\u{10440}\u{10441}\u{10442}\u{10443}\u{10444}\u{10445}\u{10446}\u{10447}\u{10448}\u{10449}\u{1044a}\u{1044b}\u{1044c}\u{1044d}\u{1044e}\u{1044f}\u{10450}\u{104b0}\u{104b1}\u{104b2}\u{104b3}\u{104b4}\u{104b5}\u{104b6}\u{104b7}\u{104b8}\u{104b9}\u{104ba}\u{104bb}\u{104bc}\u{104bd}\u{104be}\u{104bf}\u{104c0}\u{104c1}\u{104c2}\u{104c3}\u{104c4}\u{104c5}\u{104c6}\u{104c7}\u{104c8}\u{104c9}\u{104ca}\u{104cb}\u{104cc}\u{104cd}\u{104ce}\u{104cf}\u{104d0}\u{104d1}\u{104d2}\u{104d3}\u{104d7}\u{104d8}\u{104d9}\u{104da}\u{104db}\u{104dc}\u{104dd}\u{104de}\u{104df}\u{104e0}\u{104e1}\u{104e2}\u{104e3}\u{104e4}\u{104e5}\u{104e6}\u{104e7}\u{104e8}\u{104e9}\u{104ea}\u{104eb}\u{104ec}\u{104ed}\u{104ee}\u{104ef}\u{104f0}\u{104f1}\u{104f2}\u{104f3}\u{104f4}\u{104f5}\u{104f6}\u{104f7}\u{104f8}\u{104f9}\u{104fa}\u{104fb}\u{104fc}\u{10570}\u{10571}\u{10572}\u{10573}\u{10574}\u{10575}\u{10576}\u{10577}\u{10578}\u{10579}\u{1057a}\u{1057c}\u{1057d}\u{1057e}\u{1057f}\u{10580}\u{10581}\u{10582}\u{10583}\u{10584}\u{10585}\u{10586}\u{10587}\u{10588}\u{10589}\u{1058a}\u{1058c}\u{1058d}\u{1058e}\u{1058f}\u{10590}\u{10591}\u{10592}\u{10594}\u{10595}\u{10596}\u{10597}\u{10598}\u{10599}\u{1059a}\u{1059b}\u{1059c}\u{1059d}\u{1059e}\u{1059f}\u{105a0}\u{105a1}\u{105a2}\u{105a3}\u{105a4}\u{105a5}\u{105a6}\u{105a7}\u{105a8}\u{105a9}\u{105aa}\u{105ab}\u{105ac}\u{105ad}\u{105ae}\u{105af}\u{105b0}\u{105b1}\u{105b2}\u{105b3}\u{105b4}\u{105b5}\u{105b6}\u{105b7}\u{105b8}\u{105b9}\u{105ba}\u{105bb}\u{105bc}\u{105bd}\u{1077f}\u{10780}\u{10781}\u{10782}\u{10783}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107c0}\u{10c80}\u{10c81}\u{10c82}\u{10c83}\u{10c84}\u{10c85}\u{10c86}\u{10c87}\u{10c88}\u{10c89}\u{10c8a}\u{10c8b}\u{10c8c}\u{10c8d}\u{10c8e}\u{10c8f}\u{10c90}\u{10c91}\u{10c92}\u{10c93}\u{10c94}\u{10c95}\u{10c96}\u{10c97}\u{10c98}\u{10c99}\u{10c9a}\u{10c9b}\u{10c9c}\u{10c9d}\u{10c9e}\u{10c9f}\u{10ca0}\u{10ca1}\u{10ca2}\u{10ca3}\u{10ca4}\u{10ca5}\u{10ca6}\u{10ca7}\u{10ca8}\u{10ca9}\u{10caa}\u{10cab}\u{10cac}\u{10cad}\u{10cae}\u{10caf}\u{10cb0}\u{10cb1}\u{10cb2}\u{10cbf}\u{10cc0}\u{10cc1}\u{10cc2}\u{10cc3}\u{10cc4}\u{10cc5}\u{10cc6}\u{10cc7}\u{10cc8}\u{10cc9}\u{10cca}\u{10ccb}\u{10ccc}\u{10ccd}\u{10cce}\u{10ccf}\u{10cd0}\u{10cd1}\u{10cd2}\u{10cd3}\u{10cd4}\u{10cd5}\u{10cd6}\u{10cd7}\u{10cd8}\u{10cd9}\u{10cda}\u{10cdb}\u{10cdc}\u{10cdd}\u{10cde}\u{10cdf}\u{10ce0}\u{10ce1}\u{10ce2}\u{10ce3}\u{10ce4}\u{10ce5}\u{10ce6}\u{10ce7}\u{10ce8}\u{10ce9}\u{10cea}\u{10ceb}\u{10cec}\u{10ced}\u{10cee}\u{10cef}\u{10cf0}\u{10cf1}\u{10cf2}\u{10cf3}\u{10d50}\u{10d51}\u{10d52}\u{10d53}\u{10d54}\u{10d55}\u{10d56}\u{10d57}\u{10d58}\u{10d59}\u{10d5a}\u{10d5b}\u{10d5c}\u{10d5d}\u{10d5e}\u{10d5f}\u{10d60}\u{10d61}\u{10d62}\u{10d63}\u{10d64}\u{10d65}\u{10d6f}\u{10d70}\u{10d71}\u{10d72}\u{10d73}\u{10d74}\u{10d75}\u{10d76}\u{10d77}\u{10d78}\u{10d79}\u{10d7a}\u{10d7b}\u{10d7c}\u{10d7d}\u{10d7e}\u{10d7f}\u{10d80}\u{10d81}\u{10d82}\u{10d83}\u{10d84}\u{10d85}\u{10d86}\u{118a0}\u{118a1}\u{118a2}\u{118a3}\u{118a4}\u{118a5}\u{118a6}\u{118a7}\u{118a8}\u{118a9}\u{118aa}\u{118ab}\u{118ac}\u{118ad}\u{118ae}\u{118af}\u{118b0}\u{118b1}\u{118b2}\u{118b3}\u{118b4}\u{118b5}\u{118b6}\u{118b7}\u{118b8}\u{118b9}\u{118ba}\u{118bb}\u{118bc}\u{118bd}\u{118be}\u{118bf}\u{118c0}\u{118c1}\u{118c2}\u{118c3}\u{118c4}\u{118c5}\u{118c6}\u{118c7}\u{118c8}\u{118c9}\u{118ca}\u{118cb}\u{118cc}\u{118cd}\u{118ce}\u{118cf}\u{118d0}\u{118d1}\u{118d2}\u{118d3}\u{118d4}\u{118d5}\u{118d6}\u{118d7}\u{118d8}\u{118d9}\u{118da}\u{118db}\u{118dc}\u{118dd}\u{118de}\u{118df}\u{118e0}\u{16e40}\u{16e41}\u{16e42}\u{16e43}\u{16e44}\u{16e45}\u{16e46}\u{16e47}\u{16e48}\u{16e49}\u{16e4a}\u{16e4b}\u{16e4c}\u{16e4d}\u{16e4e}\u{16e4f}\u{16e50}\u{16e51}\u{16e52}\u{16e53}\u{16e54}\u{16e55}\u{16e56}\u{16e57}\u{16e58}\u{16e59}\u{16e5a}\u{16e5b}\u{16e5c}\u{16e5d}\u{16e5e}\u{16e5f}\u{16e60}\u{16e61}\u{16e62}\u{16e63}\u{16e64}\u{16e65}\u{16e66}\u{16e67}\u{16e68}\u{16e69}\u{16e6a}\u{16e6b}\u{16e6c}\u{16e6d}\u{16e6e}\u{16e6f}\u{16e70}\u{16e71}\u{16e72}\u{16e73}\u{16e74}\u{16e75}\u{16e76}\u{16e77}\u{16e78}\u{16e79}\u{16e7a}\u{16e7b}\u{16e7c}\u{16e7d}\u{16e7e}\u{16e7f}\u{16e80}\u{16ea0}\u{16ea1}\u{16ea2}\u{16ea3}\u{16ea4}\u{16ea5}\u{16ea6}\u{16ea7}\u{16ea8}\u{16ea9}\u{16eaa}\u{16eab}\u{16eac}\u{16ead}\u{16eae}\u{16eaf}\u{16eb0}\u{16eb1}\u{16eb2}\u{16eb3}\u{16eb4}\u{16eb5}\u{16eb6}\u{16eb7}\u{16eb8}\u{16eba}\u{16ebb}\u{16ebc}\u{16ebd}\u{16ebe}\u{16ebf}\u{16ec0}\u{16ec1}\u{16ec2}\u{16ec3}\u{16ec4}\u{16ec5}\u{16ec6}\u{16ec7}\u{16ec8}\u{16ec9}\u{16eca}\u{16ecb}\u{16ecc}\u{16ecd}\u{16ece}\u{16ecf}\u{16ed0}\u{16ed1}\u{16ed2}\u{16ed3}\u{16ed4}\u{1d419}\u{1d41a}\u{1d433}\u{1d434}\u{1d44d}\u{1d44e}\u{1d454}\u{1d455}\u{1d456}\u{1d467}\u{1d468}\u{1d481}\u{1d482}\u{1d49b}\u{1d49c}\u{1d4b5}\u{1d4b6}\u{1d4b9}\u{1d4ba}\u{1d4bb}\u{1d4bc}\u{1d4bd}\u{1d4c3}\u{1d4c4}\u{1d4c5}\u{1d4cf}\u{1d4d0}\u{1d4e9}\u{1d4ea}\u{1d503}\u{1d504}\u{1d51d}\u{1d51e}\u{1d537}\u{1d538}\u{1d551}\u{1d552}\u{1d56b}\u{1d56c}\u{1d585}\u{1d586}\u{1d59f}\u{1d5a0}\u{1d5b9}\u{1d5ba}\u{1d5d3}\u{1d5d4}\u{1d5ed}\u{1d5ee}\u{1d607}\u{1d608}\u{1d621}\u{1d622}\u{1d63b}\u{1d63c}\u{1d655}\u{1d656}\u{1d66f}\u{1d670}\u{1d689}\u{1d68a}\u{1d6a7}\u{1d6c1}\u{1d6c2}\u{1d6da}\u{1d6db}\u{1d6dc}\u{1d6e1}\u{1d6e2}\u{1d6fb}\u{1d6fc}\u{1d714}\u{1d715}\u{1d716}\u{1d71b}\u{1d71c}\u{1d735}\u{1d736}\u{1d74e}\u{1d74f}\u{1d750}\u{1d755}\u{1d756}\u{1d76f}\u{1d770}\u{1d788}\u{1d789}\u{1d78a}\u{1d78f}\u{1d790}\u{1d7a9}\u{1d7aa}\u{1d7c2}\u{1d7c3}\u{1d7c4}\u{1d7c9}\u{1d7ca}\u{1d7cb}\u{1d7cc}\u{1deff}\u{1df00}\u{1df09}\u{1df0a}\u{1df0b}\u{1df80}\u{1df8f}\u{1df97}\u{1dfcc}\u{1e000}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e900}\u{1e901}\u{1e902}\u{1e903}\u{1e904}\u{1e905}\u{1e906}\u{1e907}\u{1e908}\u{1e909}\u{1e90a}\u{1e90b}\u{1e90c}\u{1e90d}\u{1e90e}\u{1e90f}\u{1e910}\u{1e911}\u{1e912}\u{1e913}\u{1e914}\u{1e915}\u{1e916}\u{1e917}\u{1e918}\u{1e919}\u{1e91a}\u{1e91b}\u{1e91c}\u{1e91d}\u{1e91e}\u{1e91f}\u{1e920}\u{1e921}\u{1e922}\u{1e923}\u{1e924}\u{1e925}\u{1e926}\u{1e927}\u{1e928}\u{1e929}\u{1e92a}\u{1e92b}\u{1e92c}\u{1e92d}\u{1e92e}\u{1e92f}\u{1e930}\u{1e931}\u{1e932}\u{1e933}\u{1e934}\u{1e935}\u{1e936}\u{1e937}\u{1e938}\u{1e939}\u{1e93a}\u{1e93b}\u{1e93c}\u{1e93d}\u{1e93e}\u{1e93f}\u{1e940}\u{1e941}\u{1e942}\u{1e943}\u{1e944}\u{feff}ſK\n",
                "X\r\n\u{61}\u{2028}\u{62}\u{2029}",
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
fn unicode_lowercase_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Lowercase}}$/i{mode},n=/^\P{{Lowercase}}$/i{mode};[...'\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A\u004B\u004C\u004D\u004E\u004F\u0050\u0051\u0052\u0053\u0054\u0055\u0056\u0057\u0058\u0059\u005A\u0061\u0062\u0063\u0064\u0065\u0066\u0067\u0068\u0069\u006A\u006B\u006C\u006D\u006E\u006F\u0070\u0071\u0072\u0073\u0074\u0075\u0076\u0077\u0078\u0079\u007A\u00B5\u00C0\u00C1\u00C2\u00C3\u00C4\u00C5\u00C6\u00C7\u00C8\u00C9\u00CA\u00CB\u00CC\u00CD\u00CE\u00CF\u00D0\u00D1\u00D2\u00D3\u00D4\u00D5\u00D6\u00D8\u00D9\u00DA\u00DB\u00DC\u00DD\u00DE\u00DF\u00E0\u00E1\u00E2\u00E3\u00E4\u00E5\u00E6\u00E7\u00E8\u00E9\u00EA\u00EB\u00EC\u00ED\u00EE\u00EF\u00F0\u00F1\u00F2\u00F3\u00F4\u00F5\u00F6\u00F8\u00F9\u00FA\u00FB\u00FC\u00FD\u00FE\u00FF\u0100\u0101\u0102\u0103\u0104\u0105\u0106\u0107\u0108\u0109\u010A\u010B\u010C\u010D\u010E\u010F\u0110\u0111\u0112\u0113\u0114\u0115\u0116\u0117\u0118\u0119\u011A\u011B\u011C\u011D\u011E\u011F\u0120\u0121\u0122\u0123\u0124\u0125\u0126\u0127\u0128\u0129\u012A\u012B\u012C\u012D\u012E\u012F\u0132\u0133\u0134\u0135\u0136\u0137\u0139\u013A\u013B\u013C\u013D\u013E\u013F\u0140\u0141\u0142\u0143\u0144\u0145\u0146\u0147\u0148\u014A\u014B\u014C\u014D\u014E\u014F\u0150\u0151\u0152\u0153\u0154\u0155\u0156\u0157\u0158\u0159\u015A\u015B\u015C\u015D\u015E\u015F\u0160\u0161\u0162\u0163\u0164\u0165\u0166\u0167\u0168\u0169\u016A\u016B\u016C\u016D\u016E\u016F\u0170\u0171\u0172\u0173\u0174\u0175\u0176\u0177\u0178\u0179\u017A\u017B\u017C\u017D\u017E\u017F\u0180\u0181\u0182\u0183\u0184\u0185\u0186\u0187\u0188\u0189\u018A\u018B\u018C\u018E\u018F\u0190\u0191\u0192\u0193\u0194\u0195\u0196\u0197\u0198\u0199\u019A\u019B\u019C\u019D\u019E\u019F\u01A0\u01A1\u01A2\u01A3\u01A4\u01A5\u01A6\u01A7\u01A8\u01A9\u01AC\u01AD\u01AE\u01AF\u01B0\u01B1\u01B2\u01B3\u01B4\u01B5\u01B6\u01B7\u01B8\u01B9\u01BC\u01BD\u01BF\u01C4\u01C5\u01C6\u01C7\u01C8\u01C9\u01CA\u01CB\u01CC\u01CD\u01CE\u01CF\u01D0\u01D1\u01D2\u01D3\u01D4\u01D5\u01D6\u01D7\u01D8\u01D9\u01DA\u01DB\u01DC\u01DD\u01DE\u01DF\u01E0\u01E1\u01E2\u01E3\u01E4\u01E5\u01E6\u01E7\u01E8\u01E9\u01EA\u01EB\u01EC\u01ED\u01EE\u01EF\u01F1\u01F2\u01F3\u01F4\u01F5\u01F6\u01F7\u01F8\u01F9\u01FA\u01FB\u01FC\u01FD\u01FE\u01FF\u0200\u0201\u0202\u0203\u0204\u0205\u0206\u0207\u0208\u0209\u020A\u020B\u020C\u020D\u020E\u020F\u0210\u0211\u0212\u0213\u0214\u0215\u0216\u0217\u0218\u0219\u021A\u021B\u021C\u021D\u021E\u021F\u0220\u0222\u0223\u0224\u0225\u0226\u0227\u0228\u0229\u022A\u022B\u022C\u022D\u022E\u022F\u0230\u0231\u0232\u0233\u023A\u023B\u023C\u023D\u023E\u023F\u0240\u0241\u0242\u0243\u0244\u0245\u0246\u0247\u0248\u0249\u024A\u024B\u024C\u024D\u024E\u024F\u0250\u0251\u0252\u0253\u0254\u0256\u0257\u0259\u025B\u025C\u0260\u0261\u0263\u0264\u0265\u0266\u0268\u0269\u026A\u026B\u026C\u026F\u0271\u0272\u0275\u027D\u0280\u0282\u0283\u0287\u0288\u0289\u028A\u028B\u028C\u0292\u029D\u029E\u0345\u0370\u0371\u0372\u0373\u0376\u0377\u037B\u037C\u037D\u037F\u0386\u0388\u0389\u038A\u038C\u038E\u038F\u0391\u0392\u0393\u0394\u0395\u0396\u0397\u0398\u0399\u039A\u039B\u039C\u039D\u039E\u039F\u03A0\u03A1\u03A3\u03A4\u03A5\u03A6\u03A7\u03A8\u03A9\u03AA\u03AB\u03AC\u03AD\u03AE\u03AF\u03B1\u03B2\u03B3\u03B4\u03B5\u03B6\u03B7\u03B8\u03B9\u03BA\u03BB\u03BC\u03BD\u03BE\u03BF\u03C0\u03C1\u03C2\u03C3\u03C4\u03C5\u03C6\u03C7\u03C8\u03C9\u03CA\u03CB\u03CC\u03CD\u03CE\u03CF\u03D0\u03D1\u03D5\u03D6\u03D7\u03D8\u03D9\u03DA\u03DB\u03DC\u03DD\u03DE\u03DF\u03E0\u03E1\u03E2\u03E3\u03E4\u03E5\u03E6\u03E7\u03E8\u03E9\u03EA\u03EB\u03EC\u03ED\u03EE\u03EF\u03F0\u03F1\u03F2\u03F3\u03F4\u03F5\u03F7\u03F8\u03F9\u03FA\u03FB\u03FD\u03FE\u03FF\u0400\u0401\u0402\u0403\u0404\u0405\u0406\u0407\u0408\u0409\u040A\u040B\u040C\u040D\u040E\u040F\u0410\u0411\u0412\u0413\u0414\u0415\u0416\u0417\u0418\u0419\u041A\u041B\u041C\u041D\u041E\u041F\u0420\u0421\u0422\u0423\u0424\u0425\u0426\u0427\u0428\u0429\u042A\u042B\u042C\u042D\u042E\u042F\u0430\u0431\u0432\u0433\u0434\u0435\u0436\u0437\u0438\u0439\u043A\u043B\u043C\u043D\u043E\u043F\u0440\u0441\u0442\u0443\u0444\u0445\u0446\u0447\u0448\u0449\u044A\u044B\u044C\u044D\u044E\u044F\u0450\u0451\u0452\u0453\u0454\u0455\u0456\u0457\u0458\u0459\u045A\u045B\u045C\u045D\u045E\u045F\u0460\u0461\u0462\u0463\u0464\u0465\u0466\u0467\u0468\u0469\u046A\u046B\u046C\u046D\u046E\u046F\u0470\u0471\u0472\u0473\u0474\u0475\u0476\u0477\u0478\u0479\u047A\u047B\u047C\u047D\u047E\u047F\u0480\u0481\u048A\u048B\u048C\u048D\u048E\u048F\u0490\u0491\u0492\u0493\u0494\u0495\u0496\u0497\u0498\u0499\u049A\u049B\u049C\u049D\u049E\u049F\u04A0\u04A1\u04A2\u04A3\u04A4\u04A5\u04A6\u04A7\u04A8\u04A9\u04AA\u04AB\u04AC\u04AD\u04AE\u04AF\u04B0\u04B1\u04B2\u04B3\u04B4\u04B5\u04B6\u04B7\u04B8\u04B9\u04BA\u04BB\u04BC\u04BD\u04BE\u04BF\u04C0\u04C1\u04C2\u04C3\u04C4\u04C5\u04C6\u04C7\u04C8\u04C9\u04CA\u04CB\u04CC\u04CD\u04CE\u04CF\u04D0\u04D1\u04D2\u04D3\u04D4\u04D5\u04D6\u04D7\u04D8\u04D9\u04DA\u04DB\u04DC\u04DD\u04DE\u04DF\u04E0\u04E1\u04E2\u04E3\u04E4\u04E5\u04E6\u04E7\u04E8\u04E9\u04EA\u04EB\u04EC\u04ED\u04EE\u04EF\u04F0\u04F1\u04F2\u04F3\u04F4\u04F5\u04F6\u04F7\u04F8\u04F9\u04FA\u04FB\u04FC\u04FD\u04FE\u04FF\u0500\u0501\u0502\u0503\u0504\u0505\u0506\u0507\u0508\u0509\u050A\u050B\u050C\u050D\u050E\u050F\u0510\u0511\u0512\u0513\u0514\u0515\u0516\u0517\u0518\u0519\u051A\u051B\u051C\u051D\u051E\u051F\u0520\u0521\u0522\u0523\u0524\u0525\u0526\u0527\u0528\u0529\u052A\u052B\u052C\u052D\u052E\u052F\u0531\u0532\u0533\u0534\u0535\u0536\u0537\u0538\u0539\u053A\u053B\u053C\u053D\u053E\u053F\u0540\u0541\u0542\u0543\u0544\u0545\u0546\u0547\u0548\u0549\u054A\u054B\u054C\u054D\u054E\u054F\u0550\u0551\u0552\u0553\u0554\u0555\u0556\u0561\u0562\u0563\u0564\u0565\u0566\u0567\u0568\u0569\u056A\u056B\u056C\u056D\u056E\u056F\u0570\u0571\u0572\u0573\u0574\u0575\u0576\u0577\u0578\u0579\u057A\u057B\u057C\u057D\u057E\u057F\u0580\u0581\u0582\u0583\u0584\u0585\u0586\u10A0\u10A1\u10A2\u10A3\u10A4\u10A5\u10A6\u10A7\u10A8\u10A9\u10AA\u10AB\u10AC\u10AD\u10AE\u10AF\u10B0\u10B1\u10B2\u10B3\u10B4\u10B5\u10B6\u10B7\u10B8\u10B9\u10BA\u10BB\u10BC\u10BD\u10BE\u10BF\u10C0\u10C1\u10C2\u10C3\u10C4\u10C5\u10C7\u10CD\u10D0\u10D1\u10D2\u10D3\u10D4\u10D5\u10D6\u10D7\u10D8\u10D9\u10DA\u10DB\u10DC\u10DD\u10DE\u10DF\u10E0\u10E1\u10E2\u10E3\u10E4\u10E5\u10E6\u10E7\u10E8\u10E9\u10EA\u10EB\u10EC\u10ED\u10EE\u10EF\u10F0\u10F1\u10F2\u10F3\u10F4\u10F5\u10F6\u10F7\u10F8\u10F9\u10FA\u10FD\u10FE\u10FF\u13A0\u13A1\u13A2\u13A3\u13A4\u13A5\u13A6\u13A7\u13A8\u13A9\u13AA\u13AB\u13AC\u13AD\u13AE\u13AF\u13B0\u13B1\u13B2\u13B3\u13B4\u13B5\u13B6\u13B7\u13B8\u13B9\u13BA\u13BB\u13BC\u13BD\u13BE\u13BF\u13C0\u13C1\u13C2\u13C3\u13C4\u13C5\u13C6\u13C7\u13C8\u13C9\u13CA\u13CB\u13CC\u13CD\u13CE\u13CF\u13D0\u13D1\u13D2\u13D3\u13D4\u13D5\u13D6\u13D7\u13D8\u13D9\u13DA\u13DB\u13DC\u13DD\u13DE\u13DF\u13E0\u13E1\u13E2\u13E3\u13E4\u13E5\u13E6\u13E7\u13E8\u13E9\u13EA\u13EB\u13EC\u13ED\u13EE\u13EF\u13F0\u13F1\u13F2\u13F3\u13F4\u13F5\u13F8\u13F9\u13FA\u13FB\u13FC\u13FD\u1C80\u1C81\u1C82\u1C83\u1C84\u1C85\u1C86\u1C87\u1C88\u1C89\u1C8A\u1C90\u1C91\u1C92\u1C93\u1C94\u1C95\u1C96\u1C97\u1C98\u1C99\u1C9A\u1C9B\u1C9C\u1C9D\u1C9E\u1C9F\u1CA0\u1CA1\u1CA2\u1CA3\u1CA4\u1CA5\u1CA6\u1CA7\u1CA8\u1CA9\u1CAA\u1CAB\u1CAC\u1CAD\u1CAE\u1CAF\u1CB0\u1CB1\u1CB2\u1CB3\u1CB4\u1CB5\u1CB6\u1CB7\u1CB8\u1CB9\u1CBA\u1CBD\u1CBE\u1CBF\u1D79\u1D7D\u1D8E\u1E00\u1E01\u1E02\u1E03\u1E04\u1E05\u1E06\u1E07\u1E08\u1E09\u1E0A\u1E0B\u1E0C\u1E0D\u1E0E\u1E0F\u1E10\u1E11\u1E12\u1E13\u1E14\u1E15\u1E16\u1E17\u1E18\u1E19\u1E1A\u1E1B\u1E1C\u1E1D\u1E1E\u1E1F\u1E20\u1E21\u1E22\u1E23\u1E24\u1E25\u1E26\u1E27\u1E28\u1E29\u1E2A\u1E2B\u1E2C\u1E2D\u1E2E\u1E2F\u1E30\u1E31\u1E32\u1E33\u1E34\u1E35\u1E36\u1E37\u1E38\u1E39\u1E3A\u1E3B\u1E3C\u1E3D\u1E3E\u1E3F\u1E40\u1E41\u1E42\u1E43\u1E44\u1E45\u1E46\u1E47\u1E48\u1E49\u1E4A\u1E4B\u1E4C\u1E4D\u1E4E\u1E4F\u1E50\u1E51\u1E52\u1E53\u1E54\u1E55\u1E56\u1E57\u1E58\u1E59\u1E5A\u1E5B\u1E5C\u1E5D\u1E5E\u1E5F\u1E60\u1E61\u1E62\u1E63\u1E64\u1E65\u1E66\u1E67\u1E68\u1E69\u1E6A\u1E6B\u1E6C\u1E6D\u1E6E\u1E6F\u1E70\u1E71\u1E72\u1E73\u1E74\u1E75\u1E76\u1E77\u1E78\u1E79\u1E7A\u1E7B\u1E7C\u1E7D\u1E7E\u1E7F\u1E80\u1E81\u1E82\u1E83\u1E84\u1E85\u1E86\u1E87\u1E88\u1E89\u1E8A\u1E8B\u1E8C\u1E8D\u1E8E\u1E8F\u1E90\u1E91\u1E92\u1E93\u1E94\u1E95\u1E9B\u1E9E\u1EA0\u1EA1\u1EA2\u1EA3\u1EA4\u1EA5\u1EA6\u1EA7\u1EA8\u1EA9\u1EAA\u1EAB\u1EAC\u1EAD\u1EAE\u1EAF\u1EB0\u1EB1\u1EB2\u1EB3\u1EB4\u1EB5\u1EB6\u1EB7\u1EB8\u1EB9\u1EBA\u1EBB\u1EBC\u1EBD\u1EBE\u1EBF\u1EC0\u1EC1\u1EC2\u1EC3\u1EC4\u1EC5\u1EC6\u1EC7\u1EC8\u1EC9\u1ECA\u1ECB\u1ECC\u1ECD\u1ECE\u1ECF\u1ED0\u1ED1\u1ED2\u1ED3\u1ED4\u1ED5\u1ED6\u1ED7\u1ED8\u1ED9\u1EDA\u1EDB\u1EDC\u1EDD\u1EDE\u1EDF\u1EE0\u1EE1\u1EE2\u1EE3\u1EE4\u1EE5\u1EE6\u1EE7\u1EE8\u1EE9\u1EEA\u1EEB\u1EEC\u1EED\u1EEE\u1EEF\u1EF0\u1EF1\u1EF2\u1EF3\u1EF4\u1EF5\u1EF6\u1EF7\u1EF8\u1EF9\u1EFA\u1EFB\u1EFC\u1EFD\u1EFE\u1EFF\u1F00\u1F01\u1F02\u1F03\u1F04\u1F05\u1F06\u1F07\u1F08\u1F09\u1F0A\u1F0B\u1F0C\u1F0D\u1F0E\u1F0F\u1F10\u1F11\u1F12\u1F13\u1F14\u1F15\u1F18\u1F19\u1F1A\u1F1B\u1F1C\u1F1D\u1F20\u1F21\u1F22\u1F23\u1F24\u1F25\u1F26\u1F27\u1F28\u1F29\u1F2A\u1F2B\u1F2C\u1F2D\u1F2E\u1F2F\u1F30\u1F31\u1F32\u1F33\u1F34\u1F35\u1F36\u1F37\u1F38\u1F39\u1F3A\u1F3B\u1F3C\u1F3D\u1F3E\u1F3F\u1F40\u1F41\u1F42\u1F43\u1F44\u1F45\u1F48\u1F49\u1F4A\u1F4B\u1F4C\u1F4D\u1F51\u1F53\u1F55\u1F57\u1F59\u1F5B\u1F5D\u1F5F\u1F60\u1F61\u1F62\u1F63\u1F64\u1F65\u1F66\u1F67\u1F68\u1F69\u1F6A\u1F6B\u1F6C\u1F6D\u1F6E\u1F6F\u1F70\u1F71\u1F72\u1F73\u1F74\u1F75\u1F76\u1F77\u1F78\u1F79\u1F7A\u1F7B\u1F7C\u1F7D\u1F80\u1F81\u1F82\u1F83\u1F84\u1F85\u1F86\u1F87\u1F88\u1F89\u1F8A\u1F8B\u1F8C\u1F8D\u1F8E\u1F8F\u1F90\u1F91\u1F92\u1F93\u1F94\u1F95\u1F96\u1F97\u1F98\u1F99\u1F9A\u1F9B\u1F9C\u1F9D\u1F9E\u1F9F\u1FA0\u1FA1\u1FA2\u1FA3\u1FA4\u1FA5\u1FA6\u1FA7\u1FA8\u1FA9\u1FAA\u1FAB\u1FAC\u1FAD\u1FAE\u1FAF\u1FB0\u1FB1\u1FB3\u1FB8\u1FB9\u1FBA\u1FBB\u1FBC\u1FBE\u1FC3\u1FC8\u1FC9\u1FCA\u1FCB\u1FCC\u1FD0\u1FD1\u1FD8\u1FD9\u1FDA\u1FDB\u1FE0\u1FE1\u1FE5\u1FE8\u1FE9\u1FEA\u1FEB\u1FEC\u1FF3\u1FF8\u1FF9\u1FFA\u1FFB\u1FFC\u2126\u212A\u212B\u2132\u214E\u2160\u2161\u2162\u2163\u2164\u2165\u2166\u2167\u2168\u2169\u216A\u216B\u216C\u216D\u216E\u216F\u2170\u2171\u2172\u2173\u2174\u2175\u2176\u2177\u2178\u2179\u217A\u217B\u217C\u217D\u217E\u217F\u2183\u2184\u24B6\u24B7\u24B8\u24B9\u24BA\u24BB\u24BC\u24BD\u24BE\u24BF\u24C0\u24C1\u24C2\u24C3\u24C4\u24C5\u24C6\u24C7\u24C8\u24C9\u24CA\u24CB\u24CC\u24CD\u24CE\u24CF\u24D0\u24D1\u24D2\u24D3\u24D4\u24D5\u24D6\u24D7\u24D8\u24D9\u24DA\u24DB\u24DC\u24DD\u24DE\u24DF\u24E0\u24E1\u24E2\u24E3\u24E4\u24E5\u24E6\u24E7\u24E8\u24E9\u2C00\u2C01\u2C02\u2C03\u2C04\u2C05\u2C06\u2C07\u2C08\u2C09\u2C0A\u2C0B\u2C0C\u2C0D\u2C0E\u2C0F\u2C10\u2C11\u2C12\u2C13\u2C14\u2C15\u2C16\u2C17\u2C18\u2C19\u2C1A\u2C1B\u2C1C\u2C1D\u2C1E\u2C1F\u2C20\u2C21\u2C22\u2C23\u2C24\u2C25\u2C26\u2C27\u2C28\u2C29\u2C2A\u2C2B\u2C2C\u2C2D\u2C2E\u2C2F\u2C30\u2C31\u2C32\u2C33\u2C34\u2C35\u2C36\u2C37\u2C38\u2C39\u2C3A\u2C3B\u2C3C\u2C3D\u2C3E\u2C3F\u2C40\u2C41\u2C42\u2C43\u2C44\u2C45\u2C46\u2C47\u2C48\u2C49\u2C4A\u2C4B\u2C4C\u2C4D\u2C4E\u2C4F\u2C50\u2C51\u2C52\u2C53\u2C54\u2C55\u2C56\u2C57\u2C58\u2C59\u2C5A\u2C5B\u2C5C\u2C5D\u2C5E\u2C5F\u2C60\u2C61\u2C62\u2C63\u2C64\u2C65\u2C66\u2C67\u2C68\u2C69\u2C6A\u2C6B\u2C6C\u2C6D\u2C6E\u2C6F\u2C70\u2C72\u2C73\u2C75\u2C76\u2C7E\u2C7F\u2C80\u2C81\u2C82\u2C83\u2C84\u2C85\u2C86\u2C87\u2C88\u2C89\u2C8A\u2C8B\u2C8C\u2C8D\u2C8E\u2C8F\u2C90\u2C91\u2C92\u2C93\u2C94\u2C95\u2C96\u2C97\u2C98\u2C99\u2C9A\u2C9B\u2C9C\u2C9D\u2C9E\u2C9F\u2CA0\u2CA1\u2CA2\u2CA3\u2CA4\u2CA5\u2CA6\u2CA7\u2CA8\u2CA9\u2CAA\u2CAB\u2CAC\u2CAD\u2CAE\u2CAF\u2CB0\u2CB1\u2CB2\u2CB3\u2CB4\u2CB5\u2CB6\u2CB7\u2CB8\u2CB9\u2CBA\u2CBB\u2CBC\u2CBD\u2CBE\u2CBF\u2CC0\u2CC1\u2CC2\u2CC3\u2CC4\u2CC5\u2CC6\u2CC7\u2CC8\u2CC9\u2CCA\u2CCB\u2CCC\u2CCD\u2CCE\u2CCF\u2CD0\u2CD1\u2CD2\u2CD3\u2CD4\u2CD5\u2CD6\u2CD7\u2CD8\u2CD9\u2CDA\u2CDB\u2CDC\u2CDD\u2CDE\u2CDF\u2CE0\u2CE1\u2CE2\u2CE3\u2CEB\u2CEC\u2CED\u2CEE\u2CF2\u2CF3\u2D00\u2D01\u2D02\u2D03\u2D04\u2D05\u2D06\u2D07\u2D08\u2D09\u2D0A\u2D0B\u2D0C\u2D0D\u2D0E\u2D0F\u2D10\u2D11\u2D12\u2D13\u2D14\u2D15\u2D16\u2D17\u2D18\u2D19\u2D1A\u2D1B\u2D1C\u2D1D\u2D1E\u2D1F\u2D20\u2D21\u2D22\u2D23\u2D24\u2D25\u2D27\u2D2D\uA640\uA641\uA642\uA643\uA644\uA645\uA646\uA647\uA648\uA649\uA64A\uA64B\uA64C\uA64D\uA64E\uA64F\uA650\uA651\uA652\uA653\uA654\uA655\uA656\uA657\uA658\uA659\uA65A\uA65B\uA65C\uA65D\uA65E\uA65F\uA660\uA661\uA662\uA663\uA664\uA665\uA666\uA667\uA668\uA669\uA66A\uA66B\uA66C\uA66D\uA680\uA681\uA682\uA683\uA684\uA685\uA686\uA687\uA688\uA689\uA68A\uA68B\uA68C\uA68D\uA68E\uA68F\uA690\uA691\uA692\uA693\uA694\uA695\uA696\uA697\uA698\uA699\uA69A\uA69B\uA722\uA723\uA724\uA725\uA726\uA727\uA728\uA729\uA72A\uA72B\uA72C\uA72D\uA72E\uA72F\uA732\uA733\uA734\uA735\uA736\uA737\uA738\uA739\uA73A\uA73B\uA73C\uA73D\uA73E\uA73F\uA740\uA741\uA742\uA743\uA744\uA745\uA746\uA747\uA748\uA749\uA74A\uA74B\uA74C\uA74D\uA74E\uA74F\uA750\uA751\uA752\uA753\uA754\uA755\uA756\uA757\uA758\uA759\uA75A\uA75B\uA75C\uA75D\uA75E\uA75F\uA760\uA761\uA762\uA763\uA764\uA765\uA766\uA767\uA768\uA769\uA76A\uA76B\uA76C\uA76D\uA76E\uA76F\uA779\uA77A\uA77B\uA77C\uA77D\uA77E\uA77F\uA780\uA781\uA782\uA783\uA784\uA785\uA786\uA787\uA78B\uA78C\uA78D\uA790\uA791\uA792\uA793\uA794\uA796\uA797\uA798\uA799\uA79A\uA79B\uA79C\uA79D\uA79E\uA79F\uA7A0\uA7A1\uA7A2\uA7A3\uA7A4\uA7A5\uA7A6\uA7A7\uA7A8\uA7A9\uA7AA\uA7AB\uA7AC\uA7AD\uA7AE\uA7B0\uA7B1\uA7B2\uA7B3\uA7B4\uA7B5\uA7B6\uA7B7\uA7B8\uA7B9\uA7BA\uA7BB\uA7BC\uA7BD\uA7BE\uA7BF\uA7C0\uA7C1\uA7C2\uA7C3\uA7C4\uA7C5\uA7C6\uA7C7\uA7C8\uA7C9\uA7CA\uA7CB\uA7CC\uA7CD\uA7CE\uA7CF\uA7D0\uA7D1\uA7D2\uA7D3\uA7D4\uA7D5\uA7D6\uA7D7\uA7D8\uA7D9\uA7DA\uA7DB\uA7DC\uA7F5\uA7F6\uAB53\uAB70\uAB71\uAB72\uAB73\uAB74\uAB75\uAB76\uAB77\uAB78\uAB79\uAB7A\uAB7B\uAB7C\uAB7D\uAB7E\uAB7F\uAB80\uAB81\uAB82\uAB83\uAB84\uAB85\uAB86\uAB87\uAB88\uAB89\uAB8A\uAB8B\uAB8C\uAB8D\uAB8E\uAB8F\uAB90\uAB91\uAB92\uAB93\uAB94\uAB95\uAB96\uAB97\uAB98\uAB99\uAB9A\uAB9B\uAB9C\uAB9D\uAB9E\uAB9F\uABA0\uABA1\uABA2\uABA3\uABA4\uABA5\uABA6\uABA7\uABA8\uABA9\uABAA\uABAB\uABAC\uABAD\uABAE\uABAF\uABB0\uABB1\uABB2\uABB3\uABB4\uABB5\uABB6\uABB7\uABB8\uABB9\uABBA\uABBB\uABBC\uABBD\uABBE\uABBF\uFF21\uFF22\uFF23\uFF24\uFF25\uFF26\uFF27\uFF28\uFF29\uFF2A\uFF2B\uFF2C\uFF2D\uFF2E\uFF2F\uFF30\uFF31\uFF32\uFF33\uFF34\uFF35\uFF36\uFF37\uFF38\uFF39\uFF3A\uFF41\uFF42\uFF43\uFF44\uFF45\uFF46\uFF47\uFF48\uFF49\uFF4A\uFF4B\uFF4C\uFF4D\uFF4E\uFF4F\uFF50\uFF51\uFF52\uFF53\uFF54\uFF55\uFF56\uFF57\uFF58\uFF59\uFF5A\uD801\uDC00\uD801\uDC01\uD801\uDC02\uD801\uDC03\uD801\uDC04\uD801\uDC05\uD801\uDC06\uD801\uDC07\uD801\uDC08\uD801\uDC09\uD801\uDC0A\uD801\uDC0B\uD801\uDC0C\uD801\uDC0D\uD801\uDC0E\uD801\uDC0F\uD801\uDC10\uD801\uDC11\uD801\uDC12\uD801\uDC13\uD801\uDC14\uD801\uDC15\uD801\uDC16\uD801\uDC17\uD801\uDC18\uD801\uDC19\uD801\uDC1A\uD801\uDC1B\uD801\uDC1C\uD801\uDC1D\uD801\uDC1E\uD801\uDC1F\uD801\uDC20\uD801\uDC21\uD801\uDC22\uD801\uDC23\uD801\uDC24\uD801\uDC25\uD801\uDC26\uD801\uDC27\uD801\uDC28\uD801\uDC29\uD801\uDC2A\uD801\uDC2B\uD801\uDC2C\uD801\uDC2D\uD801\uDC2E\uD801\uDC2F\uD801\uDC30\uD801\uDC31\uD801\uDC32\uD801\uDC33\uD801\uDC34\uD801\uDC35\uD801\uDC36\uD801\uDC37\uD801\uDC38\uD801\uDC39\uD801\uDC3A\uD801\uDC3B\uD801\uDC3C\uD801\uDC3D\uD801\uDC3E\uD801\uDC3F\uD801\uDC40\uD801\uDC41\uD801\uDC42\uD801\uDC43\uD801\uDC44\uD801\uDC45\uD801\uDC46\uD801\uDC47\uD801\uDC48\uD801\uDC49\uD801\uDC4A\uD801\uDC4B\uD801\uDC4C\uD801\uDC4D\uD801\uDC4E\uD801\uDC4F\uD801\uDCB0\uD801\uDCB1\uD801\uDCB2\uD801\uDCB3\uD801\uDCB4\uD801\uDCB5\uD801\uDCB6\uD801\uDCB7\uD801\uDCB8\uD801\uDCB9\uD801\uDCBA\uD801\uDCBB\uD801\uDCBC\uD801\uDCBD\uD801\uDCBE\uD801\uDCBF\uD801\uDCC0\uD801\uDCC1\uD801\uDCC2\uD801\uDCC3\uD801\uDCC4\uD801\uDCC5\uD801\uDCC6\uD801\uDCC7\uD801\uDCC8\uD801\uDCC9\uD801\uDCCA\uD801\uDCCB\uD801\uDCCC\uD801\uDCCD\uD801\uDCCE\uD801\uDCCF\uD801\uDCD0\uD801\uDCD1\uD801\uDCD2\uD801\uDCD3\uD801\uDCD8\uD801\uDCD9\uD801\uDCDA\uD801\uDCDB\uD801\uDCDC\uD801\uDCDD\uD801\uDCDE\uD801\uDCDF\uD801\uDCE0\uD801\uDCE1\uD801\uDCE2\uD801\uDCE3\uD801\uDCE4\uD801\uDCE5\uD801\uDCE6\uD801\uDCE7\uD801\uDCE8\uD801\uDCE9\uD801\uDCEA\uD801\uDCEB\uD801\uDCEC\uD801\uDCED\uD801\uDCEE\uD801\uDCEF\uD801\uDCF0\uD801\uDCF1\uD801\uDCF2\uD801\uDCF3\uD801\uDCF4\uD801\uDCF5\uD801\uDCF6\uD801\uDCF7\uD801\uDCF8\uD801\uDCF9\uD801\uDCFA\uD801\uDCFB\uD801\uDD70\uD801\uDD71\uD801\uDD72\uD801\uDD73\uD801\uDD74\uD801\uDD75\uD801\uDD76\uD801\uDD77\uD801\uDD78\uD801\uDD79\uD801\uDD7A\uD801\uDD7C\uD801\uDD7D\uD801\uDD7E\uD801\uDD7F\uD801\uDD80\uD801\uDD81\uD801\uDD82\uD801\uDD83\uD801\uDD84\uD801\uDD85\uD801\uDD86\uD801\uDD87\uD801\uDD88\uD801\uDD89\uD801\uDD8A\uD801\uDD8C\uD801\uDD8D\uD801\uDD8E\uD801\uDD8F\uD801\uDD90\uD801\uDD91\uD801\uDD92\uD801\uDD94\uD801\uDD95\uD801\uDD97\uD801\uDD98\uD801\uDD99\uD801\uDD9A\uD801\uDD9B\uD801\uDD9C\uD801\uDD9D\uD801\uDD9E\uD801\uDD9F\uD801\uDDA0\uD801\uDDA1\uD801\uDDA3\uD801\uDDA4\uD801\uDDA5\uD801\uDDA6\uD801\uDDA7\uD801\uDDA8\uD801\uDDA9\uD801\uDDAA\uD801\uDDAB\uD801\uDDAC\uD801\uDDAD\uD801\uDDAE\uD801\uDDAF\uD801\uDDB0\uD801\uDDB1\uD801\uDDB3\uD801\uDDB4\uD801\uDDB5\uD801\uDDB6\uD801\uDDB7\uD801\uDDB8\uD801\uDDB9\uD801\uDDBB\uD801\uDDBC\uD803\uDC80\uD803\uDC81\uD803\uDC82\uD803\uDC83\uD803\uDC84\uD803\uDC85\uD803\uDC86\uD803\uDC87\uD803\uDC88\uD803\uDC89\uD803\uDC8A\uD803\uDC8B\uD803\uDC8C\uD803\uDC8D\uD803\uDC8E\uD803\uDC8F\uD803\uDC90\uD803\uDC91\uD803\uDC92\uD803\uDC93\uD803\uDC94\uD803\uDC95\uD803\uDC96\uD803\uDC97\uD803\uDC98\uD803\uDC99\uD803\uDC9A\uD803\uDC9B\uD803\uDC9C\uD803\uDC9D\uD803\uDC9E\uD803\uDC9F\uD803\uDCA0\uD803\uDCA1\uD803\uDCA2\uD803\uDCA3\uD803\uDCA4\uD803\uDCA5\uD803\uDCA6\uD803\uDCA7\uD803\uDCA8\uD803\uDCA9\uD803\uDCAA\uD803\uDCAB\uD803\uDCAC\uD803\uDCAD\uD803\uDCAE\uD803\uDCAF\uD803\uDCB0\uD803\uDCB1\uD803\uDCB2\uD803\uDCC0\uD803\uDCC1\uD803\uDCC2\uD803\uDCC3\uD803\uDCC4\uD803\uDCC5\uD803\uDCC6\uD803\uDCC7\uD803\uDCC8\uD803\uDCC9\uD803\uDCCA\uD803\uDCCB\uD803\uDCCC\uD803\uDCCD\uD803\uDCCE\uD803\uDCCF\uD803\uDCD0\uD803\uDCD1\uD803\uDCD2\uD803\uDCD3\uD803\uDCD4\uD803\uDCD5\uD803\uDCD6\uD803\uDCD7\uD803\uDCD8\uD803\uDCD9\uD803\uDCDA\uD803\uDCDB\uD803\uDCDC\uD803\uDCDD\uD803\uDCDE\uD803\uDCDF\uD803\uDCE0\uD803\uDCE1\uD803\uDCE2\uD803\uDCE3\uD803\uDCE4\uD803\uDCE5\uD803\uDCE6\uD803\uDCE7\uD803\uDCE8\uD803\uDCE9\uD803\uDCEA\uD803\uDCEB\uD803\uDCEC\uD803\uDCED\uD803\uDCEE\uD803\uDCEF\uD803\uDCF0\uD803\uDCF1\uD803\uDCF2\uD803\uDD50\uD803\uDD51\uD803\uDD52\uD803\uDD53\uD803\uDD54\uD803\uDD55\uD803\uDD56\uD803\uDD57\uD803\uDD58\uD803\uDD59\uD803\uDD5A\uD803\uDD5B\uD803\uDD5C\uD803\uDD5D\uD803\uDD5E\uD803\uDD5F\uD803\uDD60\uD803\uDD61\uD803\uDD62\uD803\uDD63\uD803\uDD64\uD803\uDD65\uD803\uDD70\uD803\uDD71\uD803\uDD72\uD803\uDD73\uD803\uDD74\uD803\uDD75\uD803\uDD76\uD803\uDD77\uD803\uDD78\uD803\uDD79\uD803\uDD7A\uD803\uDD7B\uD803\uDD7C\uD803\uDD7D\uD803\uDD7E\uD803\uDD7F\uD803\uDD80\uD803\uDD81\uD803\uDD82\uD803\uDD83\uD803\uDD84\uD803\uDD85\uD806\uDCA0\uD806\uDCA1\uD806\uDCA2\uD806\uDCA3\uD806\uDCA4\uD806\uDCA5\uD806\uDCA6\uD806\uDCA7\uD806\uDCA8\uD806\uDCA9\uD806\uDCAA\uD806\uDCAB\uD806\uDCAC\uD806\uDCAD\uD806\uDCAE\uD806\uDCAF\uD806\uDCB0\uD806\uDCB1\uD806\uDCB2\uD806\uDCB3\uD806\uDCB4\uD806\uDCB5\uD806\uDCB6\uD806\uDCB7\uD806\uDCB8\uD806\uDCB9\uD806\uDCBA\uD806\uDCBB\uD806\uDCBC\uD806\uDCBD\uD806\uDCBE\uD806\uDCBF\uD806\uDCC0\uD806\uDCC1\uD806\uDCC2\uD806\uDCC3\uD806\uDCC4\uD806\uDCC5\uD806\uDCC6\uD806\uDCC7\uD806\uDCC8\uD806\uDCC9\uD806\uDCCA\uD806\uDCCB\uD806\uDCCC\uD806\uDCCD\uD806\uDCCE\uD806\uDCCF\uD806\uDCD0\uD806\uDCD1\uD806\uDCD2\uD806\uDCD3\uD806\uDCD4\uD806\uDCD5\uD806\uDCD6\uD806\uDCD7\uD806\uDCD8\uD806\uDCD9\uD806\uDCDA\uD806\uDCDB\uD806\uDCDC\uD806\uDCDD\uD806\uDCDE\uD806\uDCDF\uD81B\uDE40\uD81B\uDE41\uD81B\uDE42\uD81B\uDE43\uD81B\uDE44\uD81B\uDE45\uD81B\uDE46\uD81B\uDE47\uD81B\uDE48\uD81B\uDE49\uD81B\uDE4A\uD81B\uDE4B\uD81B\uDE4C\uD81B\uDE4D\uD81B\uDE4E\uD81B\uDE4F\uD81B\uDE50\uD81B\uDE51\uD81B\uDE52\uD81B\uDE53\uD81B\uDE54\uD81B\uDE55\uD81B\uDE56\uD81B\uDE57\uD81B\uDE58\uD81B\uDE59\uD81B\uDE5A\uD81B\uDE5B\uD81B\uDE5C\uD81B\uDE5D\uD81B\uDE5E\uD81B\uDE5F\uD81B\uDE60\uD81B\uDE61\uD81B\uDE62\uD81B\uDE63\uD81B\uDE64\uD81B\uDE65\uD81B\uDE66\uD81B\uDE67\uD81B\uDE68\uD81B\uDE69\uD81B\uDE6A\uD81B\uDE6B\uD81B\uDE6C\uD81B\uDE6D\uD81B\uDE6E\uD81B\uDE6F\uD81B\uDE70\uD81B\uDE71\uD81B\uDE72\uD81B\uDE73\uD81B\uDE74\uD81B\uDE75\uD81B\uDE76\uD81B\uDE77\uD81B\uDE78\uD81B\uDE79\uD81B\uDE7A\uD81B\uDE7B\uD81B\uDE7C\uD81B\uDE7D\uD81B\uDE7E\uD81B\uDE7F\uD81B\uDEA0\uD81B\uDEA1\uD81B\uDEA2\uD81B\uDEA3\uD81B\uDEA4\uD81B\uDEA5\uD81B\uDEA6\uD81B\uDEA7\uD81B\uDEA8\uD81B\uDEA9\uD81B\uDEAA\uD81B\uDEAB\uD81B\uDEAC\uD81B\uDEAD\uD81B\uDEAE\uD81B\uDEAF\uD81B\uDEB0\uD81B\uDEB1\uD81B\uDEB2\uD81B\uDEB3\uD81B\uDEB4\uD81B\uDEB5\uD81B\uDEB6\uD81B\uDEB7\uD81B\uDEB8\uD81B\uDEBB\uD81B\uDEBC\uD81B\uDEBD\uD81B\uDEBE\uD81B\uDEBF\uD81B\uDEC0\uD81B\uDEC1\uD81B\uDEC2\uD81B\uDEC3\uD81B\uDEC4\uD81B\uDEC5\uD81B\uDEC6\uD81B\uDEC7\uD81B\uDEC8\uD81B\uDEC9\uD81B\uDECA\uD81B\uDECB\uD81B\uDECC\uD81B\uDECD\uD81B\uDECE\uD81B\uDECF\uD81B\uDED0\uD81B\uDED1\uD81B\uDED2\uD81B\uDED3\uD83A\uDD00\uD83A\uDD01\uD83A\uDD02\uD83A\uDD03\uD83A\uDD04\uD83A\uDD05\uD83A\uDD06\uD83A\uDD07\uD83A\uDD08\uD83A\uDD09\uD83A\uDD0A\uD83A\uDD0B\uD83A\uDD0C\uD83A\uDD0D\uD83A\uDD0E\uD83A\uDD0F\uD83A\uDD10\uD83A\uDD11\uD83A\uDD12\uD83A\uDD13\uD83A\uDD14\uD83A\uDD15\uD83A\uDD16\uD83A\uDD17\uD83A\uDD18\uD83A\uDD19\uD83A\uDD1A\uD83A\uDD1B\uD83A\uDD1C\uD83A\uDD1D\uD83A\uDD1E\uD83A\uDD1F\uD83A\uDD20\uD83A\uDD21\uD83A\uDD22\uD83A\uDD23\uD83A\uDD24\uD83A\uDD25\uD83A\uDD26\uD83A\uDD27\uD83A\uDD28\uD83A\uDD29\uD83A\uDD2A\uD83A\uDD2B\uD83A\uDD2C\uD83A\uDD2D\uD83A\uDD2E\uD83A\uDD2F\uD83A\uDD30\uD83A\uDD31\uD83A\uDD32\uD83A\uDD33\uD83A\uDD34\uD83A\uDD35\uD83A\uDD36\uD83A\uDD37\uD83A\uDD38\uD83A\uDD39\uD83A\uDD3A\uD83A\uDD3B\uD83A\uDD3C\uD83A\uDD3D\uD83A\uDD3E\uD83A\uDD3F\uD83A\uDD40\uD83A\uDD41\uD83A\uDD42\uD83A\uDD43'].every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Lowercase}}K]/i{mode},n=/[\P{{Lowercase}}K]/i{mode},x=/[^\P{{Lowercase}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&[...'\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A\u004B\u004C\u004D\u004E\u004F\u0050\u0051\u0052\u0053\u0054\u0055\u0056\u0057\u0058\u0059\u005A\u0061\u0062\u0063\u0064\u0065\u0066\u0067\u0068\u0069\u006A\u006B\u006C\u006D\u006E\u006F\u0070\u0071\u0072\u0073\u0074\u0075\u0076\u0077\u0078\u0079\u007A\u00B5\u00C0\u00C1\u00C2\u00C3\u00C4\u00C5\u00C6\u00C7\u00C8\u00C9\u00CA\u00CB\u00CC\u00CD\u00CE\u00CF\u00D0\u00D1\u00D2\u00D3\u00D4\u00D5\u00D6\u00D8\u00D9\u00DA\u00DB\u00DC\u00DD\u00DE\u00DF\u00E0\u00E1\u00E2\u00E3\u00E4\u00E5\u00E6\u00E7\u00E8\u00E9\u00EA\u00EB\u00EC\u00ED\u00EE\u00EF\u00F0\u00F1\u00F2\u00F3\u00F4\u00F5\u00F6\u00F8\u00F9\u00FA\u00FB\u00FC\u00FD\u00FE\u00FF\u0100\u0101\u0102\u0103\u0104\u0105\u0106\u0107\u0108\u0109\u010A\u010B\u010C\u010D\u010E\u010F\u0110\u0111\u0112\u0113\u0114\u0115\u0116\u0117\u0118\u0119\u011A\u011B\u011C\u011D\u011E\u011F\u0120\u0121\u0122\u0123\u0124\u0125\u0126\u0127\u0128\u0129\u012A\u012B\u012C\u012D\u012E\u012F\u0132\u0133\u0134\u0135\u0136\u0137\u0139\u013A\u013B\u013C\u013D\u013E\u013F\u0140\u0141\u0142\u0143\u0144\u0145\u0146\u0147\u0148\u014A\u014B\u014C\u014D\u014E\u014F\u0150\u0151\u0152\u0153\u0154\u0155\u0156\u0157\u0158\u0159\u015A\u015B\u015C\u015D\u015E\u015F\u0160\u0161\u0162\u0163\u0164\u0165\u0166\u0167\u0168\u0169\u016A\u016B\u016C\u016D\u016E\u016F\u0170\u0171\u0172\u0173\u0174\u0175\u0176\u0177\u0178\u0179\u017A\u017B\u017C\u017D\u017E\u017F\u0180\u0181\u0182\u0183\u0184\u0185\u0186\u0187\u0188\u0189\u018A\u018B\u018C\u018E\u018F\u0190\u0191\u0192\u0193\u0194\u0195\u0196\u0197\u0198\u0199\u019A\u019B\u019C\u019D\u019E\u019F\u01A0\u01A1\u01A2\u01A3\u01A4\u01A5\u01A6\u01A7\u01A8\u01A9\u01AC\u01AD\u01AE\u01AF\u01B0\u01B1\u01B2\u01B3\u01B4\u01B5\u01B6\u01B7\u01B8\u01B9\u01BC\u01BD\u01BF\u01C4\u01C5\u01C6\u01C7\u01C8\u01C9\u01CA\u01CB\u01CC\u01CD\u01CE\u01CF\u01D0\u01D1\u01D2\u01D3\u01D4\u01D5\u01D6\u01D7\u01D8\u01D9\u01DA\u01DB\u01DC\u01DD\u01DE\u01DF\u01E0\u01E1\u01E2\u01E3\u01E4\u01E5\u01E6\u01E7\u01E8\u01E9\u01EA\u01EB\u01EC\u01ED\u01EE\u01EF\u01F1\u01F2\u01F3\u01F4\u01F5\u01F6\u01F7\u01F8\u01F9\u01FA\u01FB\u01FC\u01FD\u01FE\u01FF\u0200\u0201\u0202\u0203\u0204\u0205\u0206\u0207\u0208\u0209\u020A\u020B\u020C\u020D\u020E\u020F\u0210\u0211\u0212\u0213\u0214\u0215\u0216\u0217\u0218\u0219\u021A\u021B\u021C\u021D\u021E\u021F\u0220\u0222\u0223\u0224\u0225\u0226\u0227\u0228\u0229\u022A\u022B\u022C\u022D\u022E\u022F\u0230\u0231\u0232\u0233\u023A\u023B\u023C\u023D\u023E\u023F\u0240\u0241\u0242\u0243\u0244\u0245\u0246\u0247\u0248\u0249\u024A\u024B\u024C\u024D\u024E\u024F\u0250\u0251\u0252\u0253\u0254\u0256\u0257\u0259\u025B\u025C\u0260\u0261\u0263\u0264\u0265\u0266\u0268\u0269\u026A\u026B\u026C\u026F\u0271\u0272\u0275\u027D\u0280\u0282\u0283\u0287\u0288\u0289\u028A\u028B\u028C\u0292\u029D\u029E\u0345\u0370\u0371\u0372\u0373\u0376\u0377\u037B\u037C\u037D\u037F\u0386\u0388\u0389\u038A\u038C\u038E\u038F\u0391\u0392\u0393\u0394\u0395\u0396\u0397\u0398\u0399\u039A\u039B\u039C\u039D\u039E\u039F\u03A0\u03A1\u03A3\u03A4\u03A5\u03A6\u03A7\u03A8\u03A9\u03AA\u03AB\u03AC\u03AD\u03AE\u03AF\u03B1\u03B2\u03B3\u03B4\u03B5\u03B6\u03B7\u03B8\u03B9\u03BA\u03BB\u03BC\u03BD\u03BE\u03BF\u03C0\u03C1\u03C2\u03C3\u03C4\u03C5\u03C6\u03C7\u03C8\u03C9\u03CA\u03CB\u03CC\u03CD\u03CE\u03CF\u03D0\u03D1\u03D5\u03D6\u03D7\u03D8\u03D9\u03DA\u03DB\u03DC\u03DD\u03DE\u03DF\u03E0\u03E1\u03E2\u03E3\u03E4\u03E5\u03E6\u03E7\u03E8\u03E9\u03EA\u03EB\u03EC\u03ED\u03EE\u03EF\u03F0\u03F1\u03F2\u03F3\u03F4\u03F5\u03F7\u03F8\u03F9\u03FA\u03FB\u03FD\u03FE\u03FF\u0400\u0401\u0402\u0403\u0404\u0405\u0406\u0407\u0408\u0409\u040A\u040B\u040C\u040D\u040E\u040F\u0410\u0411\u0412\u0413\u0414\u0415\u0416\u0417\u0418\u0419\u041A\u041B\u041C\u041D\u041E\u041F\u0420\u0421\u0422\u0423\u0424\u0425\u0426\u0427\u0428\u0429\u042A\u042B\u042C\u042D\u042E\u042F\u0430\u0431\u0432\u0433\u0434\u0435\u0436\u0437\u0438\u0439\u043A\u043B\u043C\u043D\u043E\u043F\u0440\u0441\u0442\u0443\u0444\u0445\u0446\u0447\u0448\u0449\u044A\u044B\u044C\u044D\u044E\u044F\u0450\u0451\u0452\u0453\u0454\u0455\u0456\u0457\u0458\u0459\u045A\u045B\u045C\u045D\u045E\u045F\u0460\u0461\u0462\u0463\u0464\u0465\u0466\u0467\u0468\u0469\u046A\u046B\u046C\u046D\u046E\u046F\u0470\u0471\u0472\u0473\u0474\u0475\u0476\u0477\u0478\u0479\u047A\u047B\u047C\u047D\u047E\u047F\u0480\u0481\u048A\u048B\u048C\u048D\u048E\u048F\u0490\u0491\u0492\u0493\u0494\u0495\u0496\u0497\u0498\u0499\u049A\u049B\u049C\u049D\u049E\u049F\u04A0\u04A1\u04A2\u04A3\u04A4\u04A5\u04A6\u04A7\u04A8\u04A9\u04AA\u04AB\u04AC\u04AD\u04AE\u04AF\u04B0\u04B1\u04B2\u04B3\u04B4\u04B5\u04B6\u04B7\u04B8\u04B9\u04BA\u04BB\u04BC\u04BD\u04BE\u04BF\u04C0\u04C1\u04C2\u04C3\u04C4\u04C5\u04C6\u04C7\u04C8\u04C9\u04CA\u04CB\u04CC\u04CD\u04CE\u04CF\u04D0\u04D1\u04D2\u04D3\u04D4\u04D5\u04D6\u04D7\u04D8\u04D9\u04DA\u04DB\u04DC\u04DD\u04DE\u04DF\u04E0\u04E1\u04E2\u04E3\u04E4\u04E5\u04E6\u04E7\u04E8\u04E9\u04EA\u04EB\u04EC\u04ED\u04EE\u04EF\u04F0\u04F1\u04F2\u04F3\u04F4\u04F5\u04F6\u04F7\u04F8\u04F9\u04FA\u04FB\u04FC\u04FD\u04FE\u04FF\u0500\u0501\u0502\u0503\u0504\u0505\u0506\u0507\u0508\u0509\u050A\u050B\u050C\u050D\u050E\u050F\u0510\u0511\u0512\u0513\u0514\u0515\u0516\u0517\u0518\u0519\u051A\u051B\u051C\u051D\u051E\u051F\u0520\u0521\u0522\u0523\u0524\u0525\u0526\u0527\u0528\u0529\u052A\u052B\u052C\u052D\u052E\u052F\u0531\u0532\u0533\u0534\u0535\u0536\u0537\u0538\u0539\u053A\u053B\u053C\u053D\u053E\u053F\u0540\u0541\u0542\u0543\u0544\u0545\u0546\u0547\u0548\u0549\u054A\u054B\u054C\u054D\u054E\u054F\u0550\u0551\u0552\u0553\u0554\u0555\u0556\u0561\u0562\u0563\u0564\u0565\u0566\u0567\u0568\u0569\u056A\u056B\u056C\u056D\u056E\u056F\u0570\u0571\u0572\u0573\u0574\u0575\u0576\u0577\u0578\u0579\u057A\u057B\u057C\u057D\u057E\u057F\u0580\u0581\u0582\u0583\u0584\u0585\u0586\u10A0\u10A1\u10A2\u10A3\u10A4\u10A5\u10A6\u10A7\u10A8\u10A9\u10AA\u10AB\u10AC\u10AD\u10AE\u10AF\u10B0\u10B1\u10B2\u10B3\u10B4\u10B5\u10B6\u10B7\u10B8\u10B9\u10BA\u10BB\u10BC\u10BD\u10BE\u10BF\u10C0\u10C1\u10C2\u10C3\u10C4\u10C5\u10C7\u10CD\u10D0\u10D1\u10D2\u10D3\u10D4\u10D5\u10D6\u10D7\u10D8\u10D9\u10DA\u10DB\u10DC\u10DD\u10DE\u10DF\u10E0\u10E1\u10E2\u10E3\u10E4\u10E5\u10E6\u10E7\u10E8\u10E9\u10EA\u10EB\u10EC\u10ED\u10EE\u10EF\u10F0\u10F1\u10F2\u10F3\u10F4\u10F5\u10F6\u10F7\u10F8\u10F9\u10FA\u10FD\u10FE\u10FF\u13A0\u13A1\u13A2\u13A3\u13A4\u13A5\u13A6\u13A7\u13A8\u13A9\u13AA\u13AB\u13AC\u13AD\u13AE\u13AF\u13B0\u13B1\u13B2\u13B3\u13B4\u13B5\u13B6\u13B7\u13B8\u13B9\u13BA\u13BB\u13BC\u13BD\u13BE\u13BF\u13C0\u13C1\u13C2\u13C3\u13C4\u13C5\u13C6\u13C7\u13C8\u13C9\u13CA\u13CB\u13CC\u13CD\u13CE\u13CF\u13D0\u13D1\u13D2\u13D3\u13D4\u13D5\u13D6\u13D7\u13D8\u13D9\u13DA\u13DB\u13DC\u13DD\u13DE\u13DF\u13E0\u13E1\u13E2\u13E3\u13E4\u13E5\u13E6\u13E7\u13E8\u13E9\u13EA\u13EB\u13EC\u13ED\u13EE\u13EF\u13F0\u13F1\u13F2\u13F3\u13F4\u13F5\u13F8\u13F9\u13FA\u13FB\u13FC\u13FD\u1C80\u1C81\u1C82\u1C83\u1C84\u1C85\u1C86\u1C87\u1C88\u1C89\u1C8A\u1C90\u1C91\u1C92\u1C93\u1C94\u1C95\u1C96\u1C97\u1C98\u1C99\u1C9A\u1C9B\u1C9C\u1C9D\u1C9E\u1C9F\u1CA0\u1CA1\u1CA2\u1CA3\u1CA4\u1CA5\u1CA6\u1CA7\u1CA8\u1CA9\u1CAA\u1CAB\u1CAC\u1CAD\u1CAE\u1CAF\u1CB0\u1CB1\u1CB2\u1CB3\u1CB4\u1CB5\u1CB6\u1CB7\u1CB8\u1CB9\u1CBA\u1CBD\u1CBE\u1CBF\u1D79\u1D7D\u1D8E\u1E00\u1E01\u1E02\u1E03\u1E04\u1E05\u1E06\u1E07\u1E08\u1E09\u1E0A\u1E0B\u1E0C\u1E0D\u1E0E\u1E0F\u1E10\u1E11\u1E12\u1E13\u1E14\u1E15\u1E16\u1E17\u1E18\u1E19\u1E1A\u1E1B\u1E1C\u1E1D\u1E1E\u1E1F\u1E20\u1E21\u1E22\u1E23\u1E24\u1E25\u1E26\u1E27\u1E28\u1E29\u1E2A\u1E2B\u1E2C\u1E2D\u1E2E\u1E2F\u1E30\u1E31\u1E32\u1E33\u1E34\u1E35\u1E36\u1E37\u1E38\u1E39\u1E3A\u1E3B\u1E3C\u1E3D\u1E3E\u1E3F\u1E40\u1E41\u1E42\u1E43\u1E44\u1E45\u1E46\u1E47\u1E48\u1E49\u1E4A\u1E4B\u1E4C\u1E4D\u1E4E\u1E4F\u1E50\u1E51\u1E52\u1E53\u1E54\u1E55\u1E56\u1E57\u1E58\u1E59\u1E5A\u1E5B\u1E5C\u1E5D\u1E5E\u1E5F\u1E60\u1E61\u1E62\u1E63\u1E64\u1E65\u1E66\u1E67\u1E68\u1E69\u1E6A\u1E6B\u1E6C\u1E6D\u1E6E\u1E6F\u1E70\u1E71\u1E72\u1E73\u1E74\u1E75\u1E76\u1E77\u1E78\u1E79\u1E7A\u1E7B\u1E7C\u1E7D\u1E7E\u1E7F\u1E80\u1E81\u1E82\u1E83\u1E84\u1E85\u1E86\u1E87\u1E88\u1E89\u1E8A\u1E8B\u1E8C\u1E8D\u1E8E\u1E8F\u1E90\u1E91\u1E92\u1E93\u1E94\u1E95\u1E9B\u1E9E\u1EA0\u1EA1\u1EA2\u1EA3\u1EA4\u1EA5\u1EA6\u1EA7\u1EA8\u1EA9\u1EAA\u1EAB\u1EAC\u1EAD\u1EAE\u1EAF\u1EB0\u1EB1\u1EB2\u1EB3\u1EB4\u1EB5\u1EB6\u1EB7\u1EB8\u1EB9\u1EBA\u1EBB\u1EBC\u1EBD\u1EBE\u1EBF\u1EC0\u1EC1\u1EC2\u1EC3\u1EC4\u1EC5\u1EC6\u1EC7\u1EC8\u1EC9\u1ECA\u1ECB\u1ECC\u1ECD\u1ECE\u1ECF\u1ED0\u1ED1\u1ED2\u1ED3\u1ED4\u1ED5\u1ED6\u1ED7\u1ED8\u1ED9\u1EDA\u1EDB\u1EDC\u1EDD\u1EDE\u1EDF\u1EE0\u1EE1\u1EE2\u1EE3\u1EE4\u1EE5\u1EE6\u1EE7\u1EE8\u1EE9\u1EEA\u1EEB\u1EEC\u1EED\u1EEE\u1EEF\u1EF0\u1EF1\u1EF2\u1EF3\u1EF4\u1EF5\u1EF6\u1EF7\u1EF8\u1EF9\u1EFA\u1EFB\u1EFC\u1EFD\u1EFE\u1EFF\u1F00\u1F01\u1F02\u1F03\u1F04\u1F05\u1F06\u1F07\u1F08\u1F09\u1F0A\u1F0B\u1F0C\u1F0D\u1F0E\u1F0F\u1F10\u1F11\u1F12\u1F13\u1F14\u1F15\u1F18\u1F19\u1F1A\u1F1B\u1F1C\u1F1D\u1F20\u1F21\u1F22\u1F23\u1F24\u1F25\u1F26\u1F27\u1F28\u1F29\u1F2A\u1F2B\u1F2C\u1F2D\u1F2E\u1F2F\u1F30\u1F31\u1F32\u1F33\u1F34\u1F35\u1F36\u1F37\u1F38\u1F39\u1F3A\u1F3B\u1F3C\u1F3D\u1F3E\u1F3F\u1F40\u1F41\u1F42\u1F43\u1F44\u1F45\u1F48\u1F49\u1F4A\u1F4B\u1F4C\u1F4D\u1F51\u1F53\u1F55\u1F57\u1F59\u1F5B\u1F5D\u1F5F\u1F60\u1F61\u1F62\u1F63\u1F64\u1F65\u1F66\u1F67\u1F68\u1F69\u1F6A\u1F6B\u1F6C\u1F6D\u1F6E\u1F6F\u1F70\u1F71\u1F72\u1F73\u1F74\u1F75\u1F76\u1F77\u1F78\u1F79\u1F7A\u1F7B\u1F7C\u1F7D\u1F80\u1F81\u1F82\u1F83\u1F84\u1F85\u1F86\u1F87\u1F88\u1F89\u1F8A\u1F8B\u1F8C\u1F8D\u1F8E\u1F8F\u1F90\u1F91\u1F92\u1F93\u1F94\u1F95\u1F96\u1F97\u1F98\u1F99\u1F9A\u1F9B\u1F9C\u1F9D\u1F9E\u1F9F\u1FA0\u1FA1\u1FA2\u1FA3\u1FA4\u1FA5\u1FA6\u1FA7\u1FA8\u1FA9\u1FAA\u1FAB\u1FAC\u1FAD\u1FAE\u1FAF\u1FB0\u1FB1\u1FB3\u1FB8\u1FB9\u1FBA\u1FBB\u1FBC\u1FBE\u1FC3\u1FC8\u1FC9\u1FCA\u1FCB\u1FCC\u1FD0\u1FD1\u1FD8\u1FD9\u1FDA\u1FDB\u1FE0\u1FE1\u1FE5\u1FE8\u1FE9\u1FEA\u1FEB\u1FEC\u1FF3\u1FF8\u1FF9\u1FFA\u1FFB\u1FFC\u2126\u212A\u212B\u2132\u214E\u2160\u2161\u2162\u2163\u2164\u2165\u2166\u2167\u2168\u2169\u216A\u216B\u216C\u216D\u216E\u216F\u2170\u2171\u2172\u2173\u2174\u2175\u2176\u2177\u2178\u2179\u217A\u217B\u217C\u217D\u217E\u217F\u2183\u2184\u24B6\u24B7\u24B8\u24B9\u24BA\u24BB\u24BC\u24BD\u24BE\u24BF\u24C0\u24C1\u24C2\u24C3\u24C4\u24C5\u24C6\u24C7\u24C8\u24C9\u24CA\u24CB\u24CC\u24CD\u24CE\u24CF\u24D0\u24D1\u24D2\u24D3\u24D4\u24D5\u24D6\u24D7\u24D8\u24D9\u24DA\u24DB\u24DC\u24DD\u24DE\u24DF\u24E0\u24E1\u24E2\u24E3\u24E4\u24E5\u24E6\u24E7\u24E8\u24E9\u2C00\u2C01\u2C02\u2C03\u2C04\u2C05\u2C06\u2C07\u2C08\u2C09\u2C0A\u2C0B\u2C0C\u2C0D\u2C0E\u2C0F\u2C10\u2C11\u2C12\u2C13\u2C14\u2C15\u2C16\u2C17\u2C18\u2C19\u2C1A\u2C1B\u2C1C\u2C1D\u2C1E\u2C1F\u2C20\u2C21\u2C22\u2C23\u2C24\u2C25\u2C26\u2C27\u2C28\u2C29\u2C2A\u2C2B\u2C2C\u2C2D\u2C2E\u2C2F\u2C30\u2C31\u2C32\u2C33\u2C34\u2C35\u2C36\u2C37\u2C38\u2C39\u2C3A\u2C3B\u2C3C\u2C3D\u2C3E\u2C3F\u2C40\u2C41\u2C42\u2C43\u2C44\u2C45\u2C46\u2C47\u2C48\u2C49\u2C4A\u2C4B\u2C4C\u2C4D\u2C4E\u2C4F\u2C50\u2C51\u2C52\u2C53\u2C54\u2C55\u2C56\u2C57\u2C58\u2C59\u2C5A\u2C5B\u2C5C\u2C5D\u2C5E\u2C5F\u2C60\u2C61\u2C62\u2C63\u2C64\u2C65\u2C66\u2C67\u2C68\u2C69\u2C6A\u2C6B\u2C6C\u2C6D\u2C6E\u2C6F\u2C70\u2C72\u2C73\u2C75\u2C76\u2C7E\u2C7F\u2C80\u2C81\u2C82\u2C83\u2C84\u2C85\u2C86\u2C87\u2C88\u2C89\u2C8A\u2C8B\u2C8C\u2C8D\u2C8E\u2C8F\u2C90\u2C91\u2C92\u2C93\u2C94\u2C95\u2C96\u2C97\u2C98\u2C99\u2C9A\u2C9B\u2C9C\u2C9D\u2C9E\u2C9F\u2CA0\u2CA1\u2CA2\u2CA3\u2CA4\u2CA5\u2CA6\u2CA7\u2CA8\u2CA9\u2CAA\u2CAB\u2CAC\u2CAD\u2CAE\u2CAF\u2CB0\u2CB1\u2CB2\u2CB3\u2CB4\u2CB5\u2CB6\u2CB7\u2CB8\u2CB9\u2CBA\u2CBB\u2CBC\u2CBD\u2CBE\u2CBF\u2CC0\u2CC1\u2CC2\u2CC3\u2CC4\u2CC5\u2CC6\u2CC7\u2CC8\u2CC9\u2CCA\u2CCB\u2CCC\u2CCD\u2CCE\u2CCF\u2CD0\u2CD1\u2CD2\u2CD3\u2CD4\u2CD5\u2CD6\u2CD7\u2CD8\u2CD9\u2CDA\u2CDB\u2CDC\u2CDD\u2CDE\u2CDF\u2CE0\u2CE1\u2CE2\u2CE3\u2CEB\u2CEC\u2CED\u2CEE\u2CF2\u2CF3\u2D00\u2D01\u2D02\u2D03\u2D04\u2D05\u2D06\u2D07\u2D08\u2D09\u2D0A\u2D0B\u2D0C\u2D0D\u2D0E\u2D0F\u2D10\u2D11\u2D12\u2D13\u2D14\u2D15\u2D16\u2D17\u2D18\u2D19\u2D1A\u2D1B\u2D1C\u2D1D\u2D1E\u2D1F\u2D20\u2D21\u2D22\u2D23\u2D24\u2D25\u2D27\u2D2D\uA640\uA641\uA642\uA643\uA644\uA645\uA646\uA647\uA648\uA649\uA64A\uA64B\uA64C\uA64D\uA64E\uA64F\uA650\uA651\uA652\uA653\uA654\uA655\uA656\uA657\uA658\uA659\uA65A\uA65B\uA65C\uA65D\uA65E\uA65F\uA660\uA661\uA662\uA663\uA664\uA665\uA666\uA667\uA668\uA669\uA66A\uA66B\uA66C\uA66D\uA680\uA681\uA682\uA683\uA684\uA685\uA686\uA687\uA688\uA689\uA68A\uA68B\uA68C\uA68D\uA68E\uA68F\uA690\uA691\uA692\uA693\uA694\uA695\uA696\uA697\uA698\uA699\uA69A\uA69B\uA722\uA723\uA724\uA725\uA726\uA727\uA728\uA729\uA72A\uA72B\uA72C\uA72D\uA72E\uA72F\uA732\uA733\uA734\uA735\uA736\uA737\uA738\uA739\uA73A\uA73B\uA73C\uA73D\uA73E\uA73F\uA740\uA741\uA742\uA743\uA744\uA745\uA746\uA747\uA748\uA749\uA74A\uA74B\uA74C\uA74D\uA74E\uA74F\uA750\uA751\uA752\uA753\uA754\uA755\uA756\uA757\uA758\uA759\uA75A\uA75B\uA75C\uA75D\uA75E\uA75F\uA760\uA761\uA762\uA763\uA764\uA765\uA766\uA767\uA768\uA769\uA76A\uA76B\uA76C\uA76D\uA76E\uA76F\uA779\uA77A\uA77B\uA77C\uA77D\uA77E\uA77F\uA780\uA781\uA782\uA783\uA784\uA785\uA786\uA787\uA78B\uA78C\uA78D\uA790\uA791\uA792\uA793\uA794\uA796\uA797\uA798\uA799\uA79A\uA79B\uA79C\uA79D\uA79E\uA79F\uA7A0\uA7A1\uA7A2\uA7A3\uA7A4\uA7A5\uA7A6\uA7A7\uA7A8\uA7A9\uA7AA\uA7AB\uA7AC\uA7AD\uA7AE\uA7B0\uA7B1\uA7B2\uA7B3\uA7B4\uA7B5\uA7B6\uA7B7\uA7B8\uA7B9\uA7BA\uA7BB\uA7BC\uA7BD\uA7BE\uA7BF\uA7C0\uA7C1\uA7C2\uA7C3\uA7C4\uA7C5\uA7C6\uA7C7\uA7C8\uA7C9\uA7CA\uA7CB\uA7CC\uA7CD\uA7CE\uA7CF\uA7D0\uA7D1\uA7D2\uA7D3\uA7D4\uA7D5\uA7D6\uA7D7\uA7D8\uA7D9\uA7DA\uA7DB\uA7DC\uA7F5\uA7F6\uAB53\uAB70\uAB71\uAB72\uAB73\uAB74\uAB75\uAB76\uAB77\uAB78\uAB79\uAB7A\uAB7B\uAB7C\uAB7D\uAB7E\uAB7F\uAB80\uAB81\uAB82\uAB83\uAB84\uAB85\uAB86\uAB87\uAB88\uAB89\uAB8A\uAB8B\uAB8C\uAB8D\uAB8E\uAB8F\uAB90\uAB91\uAB92\uAB93\uAB94\uAB95\uAB96\uAB97\uAB98\uAB99\uAB9A\uAB9B\uAB9C\uAB9D\uAB9E\uAB9F\uABA0\uABA1\uABA2\uABA3\uABA4\uABA5\uABA6\uABA7\uABA8\uABA9\uABAA\uABAB\uABAC\uABAD\uABAE\uABAF\uABB0\uABB1\uABB2\uABB3\uABB4\uABB5\uABB6\uABB7\uABB8\uABB9\uABBA\uABBB\uABBC\uABBD\uABBE\uABBF\uFF21\uFF22\uFF23\uFF24\uFF25\uFF26\uFF27\uFF28\uFF29\uFF2A\uFF2B\uFF2C\uFF2D\uFF2E\uFF2F\uFF30\uFF31\uFF32\uFF33\uFF34\uFF35\uFF36\uFF37\uFF38\uFF39\uFF3A\uFF41\uFF42\uFF43\uFF44\uFF45\uFF46\uFF47\uFF48\uFF49\uFF4A\uFF4B\uFF4C\uFF4D\uFF4E\uFF4F\uFF50\uFF51\uFF52\uFF53\uFF54\uFF55\uFF56\uFF57\uFF58\uFF59\uFF5A\uD801\uDC00\uD801\uDC01\uD801\uDC02\uD801\uDC03\uD801\uDC04\uD801\uDC05\uD801\uDC06\uD801\uDC07\uD801\uDC08\uD801\uDC09\uD801\uDC0A\uD801\uDC0B\uD801\uDC0C\uD801\uDC0D\uD801\uDC0E\uD801\uDC0F\uD801\uDC10\uD801\uDC11\uD801\uDC12\uD801\uDC13\uD801\uDC14\uD801\uDC15\uD801\uDC16\uD801\uDC17\uD801\uDC18\uD801\uDC19\uD801\uDC1A\uD801\uDC1B\uD801\uDC1C\uD801\uDC1D\uD801\uDC1E\uD801\uDC1F\uD801\uDC20\uD801\uDC21\uD801\uDC22\uD801\uDC23\uD801\uDC24\uD801\uDC25\uD801\uDC26\uD801\uDC27\uD801\uDC28\uD801\uDC29\uD801\uDC2A\uD801\uDC2B\uD801\uDC2C\uD801\uDC2D\uD801\uDC2E\uD801\uDC2F\uD801\uDC30\uD801\uDC31\uD801\uDC32\uD801\uDC33\uD801\uDC34\uD801\uDC35\uD801\uDC36\uD801\uDC37\uD801\uDC38\uD801\uDC39\uD801\uDC3A\uD801\uDC3B\uD801\uDC3C\uD801\uDC3D\uD801\uDC3E\uD801\uDC3F\uD801\uDC40\uD801\uDC41\uD801\uDC42\uD801\uDC43\uD801\uDC44\uD801\uDC45\uD801\uDC46\uD801\uDC47\uD801\uDC48\uD801\uDC49\uD801\uDC4A\uD801\uDC4B\uD801\uDC4C\uD801\uDC4D\uD801\uDC4E\uD801\uDC4F\uD801\uDCB0\uD801\uDCB1\uD801\uDCB2\uD801\uDCB3\uD801\uDCB4\uD801\uDCB5\uD801\uDCB6\uD801\uDCB7\uD801\uDCB8\uD801\uDCB9\uD801\uDCBA\uD801\uDCBB\uD801\uDCBC\uD801\uDCBD\uD801\uDCBE\uD801\uDCBF\uD801\uDCC0\uD801\uDCC1\uD801\uDCC2\uD801\uDCC3\uD801\uDCC4\uD801\uDCC5\uD801\uDCC6\uD801\uDCC7\uD801\uDCC8\uD801\uDCC9\uD801\uDCCA\uD801\uDCCB\uD801\uDCCC\uD801\uDCCD\uD801\uDCCE\uD801\uDCCF\uD801\uDCD0\uD801\uDCD1\uD801\uDCD2\uD801\uDCD3\uD801\uDCD8\uD801\uDCD9\uD801\uDCDA\uD801\uDCDB\uD801\uDCDC\uD801\uDCDD\uD801\uDCDE\uD801\uDCDF\uD801\uDCE0\uD801\uDCE1\uD801\uDCE2\uD801\uDCE3\uD801\uDCE4\uD801\uDCE5\uD801\uDCE6\uD801\uDCE7\uD801\uDCE8\uD801\uDCE9\uD801\uDCEA\uD801\uDCEB\uD801\uDCEC\uD801\uDCED\uD801\uDCEE\uD801\uDCEF\uD801\uDCF0\uD801\uDCF1\uD801\uDCF2\uD801\uDCF3\uD801\uDCF4\uD801\uDCF5\uD801\uDCF6\uD801\uDCF7\uD801\uDCF8\uD801\uDCF9\uD801\uDCFA\uD801\uDCFB\uD801\uDD70\uD801\uDD71\uD801\uDD72\uD801\uDD73\uD801\uDD74\uD801\uDD75\uD801\uDD76\uD801\uDD77\uD801\uDD78\uD801\uDD79\uD801\uDD7A\uD801\uDD7C\uD801\uDD7D\uD801\uDD7E\uD801\uDD7F\uD801\uDD80\uD801\uDD81\uD801\uDD82\uD801\uDD83\uD801\uDD84\uD801\uDD85\uD801\uDD86\uD801\uDD87\uD801\uDD88\uD801\uDD89\uD801\uDD8A\uD801\uDD8C\uD801\uDD8D\uD801\uDD8E\uD801\uDD8F\uD801\uDD90\uD801\uDD91\uD801\uDD92\uD801\uDD94\uD801\uDD95\uD801\uDD97\uD801\uDD98\uD801\uDD99\uD801\uDD9A\uD801\uDD9B\uD801\uDD9C\uD801\uDD9D\uD801\uDD9E\uD801\uDD9F\uD801\uDDA0\uD801\uDDA1\uD801\uDDA3\uD801\uDDA4\uD801\uDDA5\uD801\uDDA6\uD801\uDDA7\uD801\uDDA8\uD801\uDDA9\uD801\uDDAA\uD801\uDDAB\uD801\uDDAC\uD801\uDDAD\uD801\uDDAE\uD801\uDDAF\uD801\uDDB0\uD801\uDDB1\uD801\uDDB3\uD801\uDDB4\uD801\uDDB5\uD801\uDDB6\uD801\uDDB7\uD801\uDDB8\uD801\uDDB9\uD801\uDDBB\uD801\uDDBC\uD803\uDC80\uD803\uDC81\uD803\uDC82\uD803\uDC83\uD803\uDC84\uD803\uDC85\uD803\uDC86\uD803\uDC87\uD803\uDC88\uD803\uDC89\uD803\uDC8A\uD803\uDC8B\uD803\uDC8C\uD803\uDC8D\uD803\uDC8E\uD803\uDC8F\uD803\uDC90\uD803\uDC91\uD803\uDC92\uD803\uDC93\uD803\uDC94\uD803\uDC95\uD803\uDC96\uD803\uDC97\uD803\uDC98\uD803\uDC99\uD803\uDC9A\uD803\uDC9B\uD803\uDC9C\uD803\uDC9D\uD803\uDC9E\uD803\uDC9F\uD803\uDCA0\uD803\uDCA1\uD803\uDCA2\uD803\uDCA3\uD803\uDCA4\uD803\uDCA5\uD803\uDCA6\uD803\uDCA7\uD803\uDCA8\uD803\uDCA9\uD803\uDCAA\uD803\uDCAB\uD803\uDCAC\uD803\uDCAD\uD803\uDCAE\uD803\uDCAF\uD803\uDCB0\uD803\uDCB1\uD803\uDCB2\uD803\uDCC0\uD803\uDCC1\uD803\uDCC2\uD803\uDCC3\uD803\uDCC4\uD803\uDCC5\uD803\uDCC6\uD803\uDCC7\uD803\uDCC8\uD803\uDCC9\uD803\uDCCA\uD803\uDCCB\uD803\uDCCC\uD803\uDCCD\uD803\uDCCE\uD803\uDCCF\uD803\uDCD0\uD803\uDCD1\uD803\uDCD2\uD803\uDCD3\uD803\uDCD4\uD803\uDCD5\uD803\uDCD6\uD803\uDCD7\uD803\uDCD8\uD803\uDCD9\uD803\uDCDA\uD803\uDCDB\uD803\uDCDC\uD803\uDCDD\uD803\uDCDE\uD803\uDCDF\uD803\uDCE0\uD803\uDCE1\uD803\uDCE2\uD803\uDCE3\uD803\uDCE4\uD803\uDCE5\uD803\uDCE6\uD803\uDCE7\uD803\uDCE8\uD803\uDCE9\uD803\uDCEA\uD803\uDCEB\uD803\uDCEC\uD803\uDCED\uD803\uDCEE\uD803\uDCEF\uD803\uDCF0\uD803\uDCF1\uD803\uDCF2\uD803\uDD50\uD803\uDD51\uD803\uDD52\uD803\uDD53\uD803\uDD54\uD803\uDD55\uD803\uDD56\uD803\uDD57\uD803\uDD58\uD803\uDD59\uD803\uDD5A\uD803\uDD5B\uD803\uDD5C\uD803\uDD5D\uD803\uDD5E\uD803\uDD5F\uD803\uDD60\uD803\uDD61\uD803\uDD62\uD803\uDD63\uD803\uDD64\uD803\uDD65\uD803\uDD70\uD803\uDD71\uD803\uDD72\uD803\uDD73\uD803\uDD74\uD803\uDD75\uD803\uDD76\uD803\uDD77\uD803\uDD78\uD803\uDD79\uD803\uDD7A\uD803\uDD7B\uD803\uDD7C\uD803\uDD7D\uD803\uDD7E\uD803\uDD7F\uD803\uDD80\uD803\uDD81\uD803\uDD82\uD803\uDD83\uD803\uDD84\uD803\uDD85\uD806\uDCA0\uD806\uDCA1\uD806\uDCA2\uD806\uDCA3\uD806\uDCA4\uD806\uDCA5\uD806\uDCA6\uD806\uDCA7\uD806\uDCA8\uD806\uDCA9\uD806\uDCAA\uD806\uDCAB\uD806\uDCAC\uD806\uDCAD\uD806\uDCAE\uD806\uDCAF\uD806\uDCB0\uD806\uDCB1\uD806\uDCB2\uD806\uDCB3\uD806\uDCB4\uD806\uDCB5\uD806\uDCB6\uD806\uDCB7\uD806\uDCB8\uD806\uDCB9\uD806\uDCBA\uD806\uDCBB\uD806\uDCBC\uD806\uDCBD\uD806\uDCBE\uD806\uDCBF\uD806\uDCC0\uD806\uDCC1\uD806\uDCC2\uD806\uDCC3\uD806\uDCC4\uD806\uDCC5\uD806\uDCC6\uD806\uDCC7\uD806\uDCC8\uD806\uDCC9\uD806\uDCCA\uD806\uDCCB\uD806\uDCCC\uD806\uDCCD\uD806\uDCCE\uD806\uDCCF\uD806\uDCD0\uD806\uDCD1\uD806\uDCD2\uD806\uDCD3\uD806\uDCD4\uD806\uDCD5\uD806\uDCD6\uD806\uDCD7\uD806\uDCD8\uD806\uDCD9\uD806\uDCDA\uD806\uDCDB\uD806\uDCDC\uD806\uDCDD\uD806\uDCDE\uD806\uDCDF\uD81B\uDE40\uD81B\uDE41\uD81B\uDE42\uD81B\uDE43\uD81B\uDE44\uD81B\uDE45\uD81B\uDE46\uD81B\uDE47\uD81B\uDE48\uD81B\uDE49\uD81B\uDE4A\uD81B\uDE4B\uD81B\uDE4C\uD81B\uDE4D\uD81B\uDE4E\uD81B\uDE4F\uD81B\uDE50\uD81B\uDE51\uD81B\uDE52\uD81B\uDE53\uD81B\uDE54\uD81B\uDE55\uD81B\uDE56\uD81B\uDE57\uD81B\uDE58\uD81B\uDE59\uD81B\uDE5A\uD81B\uDE5B\uD81B\uDE5C\uD81B\uDE5D\uD81B\uDE5E\uD81B\uDE5F\uD81B\uDE60\uD81B\uDE61\uD81B\uDE62\uD81B\uDE63\uD81B\uDE64\uD81B\uDE65\uD81B\uDE66\uD81B\uDE67\uD81B\uDE68\uD81B\uDE69\uD81B\uDE6A\uD81B\uDE6B\uD81B\uDE6C\uD81B\uDE6D\uD81B\uDE6E\uD81B\uDE6F\uD81B\uDE70\uD81B\uDE71\uD81B\uDE72\uD81B\uDE73\uD81B\uDE74\uD81B\uDE75\uD81B\uDE76\uD81B\uDE77\uD81B\uDE78\uD81B\uDE79\uD81B\uDE7A\uD81B\uDE7B\uD81B\uDE7C\uD81B\uDE7D\uD81B\uDE7E\uD81B\uDE7F\uD81B\uDEA0\uD81B\uDEA1\uD81B\uDEA2\uD81B\uDEA3\uD81B\uDEA4\uD81B\uDEA5\uD81B\uDEA6\uD81B\uDEA7\uD81B\uDEA8\uD81B\uDEA9\uD81B\uDEAA\uD81B\uDEAB\uD81B\uDEAC\uD81B\uDEAD\uD81B\uDEAE\uD81B\uDEAF\uD81B\uDEB0\uD81B\uDEB1\uD81B\uDEB2\uD81B\uDEB3\uD81B\uDEB4\uD81B\uDEB5\uD81B\uDEB6\uD81B\uDEB7\uD81B\uDEB8\uD81B\uDEBB\uD81B\uDEBC\uD81B\uDEBD\uD81B\uDEBE\uD81B\uDEBF\uD81B\uDEC0\uD81B\uDEC1\uD81B\uDEC2\uD81B\uDEC3\uD81B\uDEC4\uD81B\uDEC5\uD81B\uDEC6\uD81B\uDEC7\uD81B\uDEC8\uD81B\uDEC9\uD81B\uDECA\uD81B\uDECB\uD81B\uDECC\uD81B\uDECD\uD81B\uDECE\uD81B\uDECF\uD81B\uDED0\uD81B\uDED1\uD81B\uDED2\uD81B\uDED3\uD83A\uDD00\uD83A\uDD01\uD83A\uDD02\uD83A\uDD03\uD83A\uDD04\uD83A\uDD05\uD83A\uDD06\uD83A\uDD07\uD83A\uDD08\uD83A\uDD09\uD83A\uDD0A\uD83A\uDD0B\uD83A\uDD0C\uD83A\uDD0D\uD83A\uDD0E\uD83A\uDD0F\uD83A\uDD10\uD83A\uDD11\uD83A\uDD12\uD83A\uDD13\uD83A\uDD14\uD83A\uDD15\uD83A\uDD16\uD83A\uDD17\uD83A\uDD18\uD83A\uDD19\uD83A\uDD1A\uD83A\uDD1B\uD83A\uDD1C\uD83A\uDD1D\uD83A\uDD1E\uD83A\uDD1F\uD83A\uDD20\uD83A\uDD21\uD83A\uDD22\uD83A\uDD23\uD83A\uDD24\uD83A\uDD25\uD83A\uDD26\uD83A\uDD27\uD83A\uDD28\uD83A\uDD29\uD83A\uDD2A\uD83A\uDD2B\uD83A\uDD2C\uD83A\uDD2D\uD83A\uDD2E\uD83A\uDD2F\uD83A\uDD30\uD83A\uDD31\uD83A\uDD32\uD83A\uDD33\uD83A\uDD34\uD83A\uDD35\uD83A\uDD36\uD83A\uDD37\uD83A\uDD38\uD83A\uDD39\uD83A\uDD3A\uD83A\uDD3B\uD83A\uDD3C\uD83A\uDD3D\uD83A\uDD3E\uD83A\uDD3F\uD83A\uDD40\uD83A\uDD41\uD83A\uDD42\uD83A\uDD43'].every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'||c==='K'||c==='k'||c==='K')&&x.test(c)===('{mode}'==='v'&&c!=='K'&&c!=='k'&&c!=='K'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Lowercase}}/{mode},b=/\p{{Lower}}/i{mode},n=/\P{{Lower}}/i{mode};a.test('\u0061')&&a.test('\u0062')&&b.test('\u0061')&&!a.test('\u200D')&&!b.test('\u005f')&&b.test('ſ')&&b.test('K')&&n.test('\u0061')===('{mode}'==='u')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Lowercase}}/{mode},s=/\s/{mode};p.test('\u0061')&&p.test('\u0062')&&!s.test('\u0061')&&!s.test('\u0062')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Lower}}K]/i{mode},b=/[^\P{{Lowercase}}K]/i{mode},c=/[\p{{Lower}}\P{{Lowercase}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0061')&&a.test('ſ')&&b.test('\u0062')===('{mode}'==='v')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Lower}}$/{mode},b=/^\P{{Lower}}$/{mode};a.test('\u0061')&&!a.test('\u0062\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_lowercase_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Lowercase}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD835\uDC1A');m.index===0&&m[0]==='\uD835\uDC1A'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Lower}}K]))$/dim{mode}g,m=r.exec('\u200D\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Lower}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0061');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Lower}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0061');m.index===2&&m[0]==='\u0061'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Lower}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0061')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_lowercase_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0062\uDB40\uDC01\u0061\u200D'.matchAll(/(?<x>\P{{Lower}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u200D'"
        ));
        check(&format!(
            r"'\u0062\u0061\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Lowercase}})/{mode}g,'<$1:$<x>>')==='<\u0062:\u0062><\u0061:\u0061>\uDB40\uDC01\u200D'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u0062\u0061\u200D'.split(/(\p{{Lower}})/{mode}).join('|')==='\uDB40\uDC01|\u0062||\u0061|\u200D'&&'\uDB40\uDC01\u0061'.search(/\p{{Lower}}/{mode})===2&&'\u0062\u0061\uDB40\uDC01'.match(/\p{{Lower}}/{mode}g).join('')==='\u0062\u0061'"
        ));
        check(&format!(
            r"let t='';let x='\u0061\u0062\uDB40\uDC01\u0061'.replace(/(\p{{Lower}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_lowercase_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Lowercase})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Lowercase})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0061').groups.x==='\u0061'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Lower}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Lower}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0061')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Lower}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u0061')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_lowercase_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Lower}/ug,marker=0;r.lastIndex=1")
        .unwrap();
    realm
        .eval("let text='\\uDB40\\uDC01'.repeat(10000)")
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
    check(r"/[\p{Lower}&&A]/v.exec('A')===null");
    check(r"/[\p{Lower}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x61),m=/[\p{Lower}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Lower}+b/u.exec('AA')", r"/\p{Lower}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_lowercase_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\b(?<x>\p{{Lower}})+\b/d{mode}g,m=r.exec('\uDB40\uDC01\u0061\u0062\u0062\uFEFF');m.index===2&&m[0]==='\u0061\u0062\u0062'&&m.groups.x==='\u0062'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_lowercase_property_unicode18_addition() {
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u0277')===true&&n.test('\u0277')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u0277')===true&&n.test('\u0277')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u0277')===true&&n.test('\u0277')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u027C')===true&&n.test('\u027C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u027C')===true&&n.test('\u027C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u027C')===true&&n.test('\u027C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u0558')===true&&n.test('\u0558')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u058B')===true&&n.test('\u058B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u058C')===true&&n.test('\u058C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u209D')===true&&n.test('\u209D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u209E')===true&&n.test('\u209E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\u209F')===true&&n.test('\u209F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uA7DD')===false&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uA7DD')===true&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uA7DD')===false&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uA7DD')===true&&n.test('\uA7DD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uA7DD')===false&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uA7DD')===true&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uA7DD')===false&&n.test('\uA7DD')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uA7DD')===true&&n.test('\uA7DD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uA7E2')===false&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uA7E2')===true&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uA7E2')===false&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uA7E2')===true&&n.test('\uA7E2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uA7E2')===false&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uA7E2')===true&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uA7E2')===false&&n.test('\uA7E2')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uA7E2')===true&&n.test('\uA7E2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uAB4B')===true&&n.test('\uAB4B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uAB4B')===true&&n.test('\uAB4B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uAB4B')===true&&n.test('\uAB4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uAB4C')===true&&n.test('\uAB4C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uAB4C')===true&&n.test('\uAB4C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uAB4C')===true&&n.test('\uAB4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uAB6C')===false&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uAB6C')===true&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uAB6C')===false&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uAB6C')===true&&n.test('\uAB6C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uAB6C')===false&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uAB6C')===true&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uAB6C')===false&&n.test('\uAB6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uAB6C')===true&&n.test('\uAB6C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uAB6D')===false&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uAB6D')===true&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uAB6D')===false&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uAB6D')===true&&n.test('\uAB6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uAB6D')===false&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uAB6D')===true&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uAB6D')===false&&n.test('\uAB6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uAB6D')===true&&n.test('\uAB6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD801\uDFBB')===true&&n.test('\uD801\uDFBB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD801\uDFBC')===true&&n.test('\uD801\uDFBC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD801\uDFBD')===true&&n.test('\uD801\uDFBD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD801\uDFBE')===true&&n.test('\uD801\uDFBE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD801\uDFBF')===true&&n.test('\uD801\uDFBF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD835\uDEA6')===true&&n.test('\uD835\uDEA6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF1F')===true&&n.test('\uD837\uDF1F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF20')===true&&n.test('\uD837\uDF20')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF21')===true&&n.test('\uD837\uDF21')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF22')===true&&n.test('\uD837\uDF22')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF23')===true&&n.test('\uD837\uDF23')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF24')===true&&n.test('\uD837\uDF24')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF2B')===true&&n.test('\uD837\uDF2B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF2C')===true&&n.test('\uD837\uDF2C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF2D')===true&&n.test('\uD837\uDF2D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF2E')===true&&n.test('\uD837\uDF2E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF2F')===true&&n.test('\uD837\uDF2F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF30')===true&&n.test('\uD837\uDF30')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF31')===true&&n.test('\uD837\uDF31')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF32')===true&&n.test('\uD837\uDF32')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF33')===true&&n.test('\uD837\uDF33')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF34')===true&&n.test('\uD837\uDF34')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF35')===true&&n.test('\uD837\uDF35')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF36')===true&&n.test('\uD837\uDF36')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF37')===true&&n.test('\uD837\uDF37')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF38')===true&&n.test('\uD837\uDF38')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF39')===true&&n.test('\uD837\uDF39')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3A')===true&&n.test('\uD837\uDF3A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3B')===true&&n.test('\uD837\uDF3B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3C')===true&&n.test('\uD837\uDF3C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3D')===true&&n.test('\uD837\uDF3D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3E')===true&&n.test('\uD837\uDF3E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF3F')===true&&n.test('\uD837\uDF3F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF40')===false&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF40')===true&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF40')===false&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF40')===true&&n.test('\uD837\uDF40')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF40')===false&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF40')===true&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF40')===false&&n.test('\uD837\uDF40')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF40')===true&&n.test('\uD837\uDF40')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF41')===true&&n.test('\uD837\uDF41')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF42')===true&&n.test('\uD837\uDF42')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF43')===true&&n.test('\uD837\uDF43')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF44')===true&&n.test('\uD837\uDF44')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF45')===true&&n.test('\uD837\uDF45')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF46')===true&&n.test('\uD837\uDF46')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF47')===true&&n.test('\uD837\uDF47')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF48')===false&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF48')===true&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF48')===false&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF48')===true&&n.test('\uD837\uDF48')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF48')===false&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF48')===true&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF48')===false&&n.test('\uD837\uDF48')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF48')===true&&n.test('\uD837\uDF48')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF49')===true&&n.test('\uD837\uDF49')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4A')===false&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4A')===true&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4A')===false&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4A')===true&&n.test('\uD837\uDF4A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4A')===false&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4A')===true&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4A')===false&&n.test('\uD837\uDF4A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4A')===true&&n.test('\uD837\uDF4A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4B')===true&&n.test('\uD837\uDF4B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4C')===true&&n.test('\uD837\uDF4C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4D')===false&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4D')===true&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4D')===false&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4D')===true&&n.test('\uD837\uDF4D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4D')===false&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4D')===true&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4D')===false&&n.test('\uD837\uDF4D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4D')===true&&n.test('\uD837\uDF4D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4E')===true&&n.test('\uD837\uDF4E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF4F')===true&&n.test('\uD837\uDF4F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF50')===true&&n.test('\uD837\uDF50')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF51')===false&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF51')===true&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF51')===false&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF51')===true&&n.test('\uD837\uDF51')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF51')===false&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF51')===true&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF51')===false&&n.test('\uD837\uDF51')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF51')===true&&n.test('\uD837\uDF51')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF52')===true&&n.test('\uD837\uDF52')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF53')===true&&n.test('\uD837\uDF53')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF54')===true&&n.test('\uD837\uDF54')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF55')===true&&n.test('\uD837\uDF55')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF56')===true&&n.test('\uD837\uDF56')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF57')===true&&n.test('\uD837\uDF57')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF58')===true&&n.test('\uD837\uDF58')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF59')===true&&n.test('\uD837\uDF59')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5A')===true&&n.test('\uD837\uDF5A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5B')===true&&n.test('\uD837\uDF5B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5C')===true&&n.test('\uD837\uDF5C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5D')===true&&n.test('\uD837\uDF5D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5E')===true&&n.test('\uD837\uDF5E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF5F')===true&&n.test('\uD837\uDF5F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF60')===true&&n.test('\uD837\uDF60')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF61')===true&&n.test('\uD837\uDF61')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF62')===true&&n.test('\uD837\uDF62')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF63')===true&&n.test('\uD837\uDF63')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF64')===true&&n.test('\uD837\uDF64')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF65')===true&&n.test('\uD837\uDF65')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF66')===true&&n.test('\uD837\uDF66')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF67')===true&&n.test('\uD837\uDF67')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF68')===false&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF68')===true&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF68')===false&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF68')===true&&n.test('\uD837\uDF68')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF68')===false&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF68')===true&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF68')===false&&n.test('\uD837\uDF68')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF68')===true&&n.test('\uD837\uDF68')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF69')===true&&n.test('\uD837\uDF69')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6A')===false&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6A')===true&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6A')===false&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6A')===true&&n.test('\uD837\uDF6A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6A')===false&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6A')===true&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6A')===false&&n.test('\uD837\uDF6A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6A')===true&&n.test('\uD837\uDF6A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6B')===true&&n.test('\uD837\uDF6B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6C')===false&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6C')===true&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6C')===false&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6C')===true&&n.test('\uD837\uDF6C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6C')===false&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6C')===true&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6C')===false&&n.test('\uD837\uDF6C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6C')===true&&n.test('\uD837\uDF6C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6D')===true&&n.test('\uD837\uDF6D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6E')===false&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6E')===true&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6E')===false&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6E')===true&&n.test('\uD837\uDF6E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6E')===false&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6E')===true&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6E')===false&&n.test('\uD837\uDF6E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6E')===true&&n.test('\uD837\uDF6E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF6F')===true&&n.test('\uD837\uDF6F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF70')===true&&n.test('\uD837\uDF70')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF71')===true&&n.test('\uD837\uDF71')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF72')===false&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF72')===true&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF72')===false&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF72')===true&&n.test('\uD837\uDF72')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF72')===false&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF72')===true&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF72')===false&&n.test('\uD837\uDF72')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF72')===true&&n.test('\uD837\uDF72')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF73')===true&&n.test('\uD837\uDF73')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF74')===false&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF74')===true&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF74')===false&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF74')===true&&n.test('\uD837\uDF74')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF74')===false&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF74')===true&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF74')===false&&n.test('\uD837\uDF74')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF74')===true&&n.test('\uD837\uDF74')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF75')===true&&n.test('\uD837\uDF75')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF76')===false&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF76')===true&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF76')===false&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF76')===true&&n.test('\uD837\uDF76')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF76')===false&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF76')===true&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF76')===false&&n.test('\uD837\uDF76')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF76')===true&&n.test('\uD837\uDF76')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF77')===true&&n.test('\uD837\uDF77')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF78')===false&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF78')===true&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF78')===false&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF78')===true&&n.test('\uD837\uDF78')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF78')===false&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF78')===true&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF78')===false&&n.test('\uD837\uDF78')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF78')===true&&n.test('\uD837\uDF78')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF79')===true&&n.test('\uD837\uDF79')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7A')===false&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7A')===true&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7A')===false&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7A')===true&&n.test('\uD837\uDF7A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7A')===false&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7A')===true&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7A')===false&&n.test('\uD837\uDF7A')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7A')===true&&n.test('\uD837\uDF7A')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7B')===true&&n.test('\uD837\uDF7B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7C')===false&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7C')===true&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7C')===false&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7C')===true&&n.test('\uD837\uDF7C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7C')===false&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7C')===true&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7C')===false&&n.test('\uD837\uDF7C')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7C')===true&&n.test('\uD837\uDF7C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7D')===true&&n.test('\uD837\uDF7D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7E')===false&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7E')===true&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7E')===false&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7E')===true&&n.test('\uD837\uDF7E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7E')===false&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7E')===true&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7E')===false&&n.test('\uD837\uDF7E')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7E')===true&&n.test('\uD837\uDF7E')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF7F')===true&&n.test('\uD837\uDF7F')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF90')===true&&n.test('\uD837\uDF90')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF91')===true&&n.test('\uD837\uDF91')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF92')===true&&n.test('\uD837\uDF92')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF93')===true&&n.test('\uD837\uDF93')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF94')===true&&n.test('\uD837\uDF94')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===true",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF95')===true&&n.test('\uD837\uDF95')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDF96')===true&&n.test('\uD837\uDF96')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFCD')===true&&n.test('\uD837\uDFCD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFCE')===true&&n.test('\uD837\uDFCE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFCF')===true&&n.test('\uD837\uDFCF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD0')===true&&n.test('\uD837\uDFD0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD1')===true&&n.test('\uD837\uDFD1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD2')===true&&n.test('\uD837\uDFD2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD3')===true&&n.test('\uD837\uDFD3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD4')===true&&n.test('\uD837\uDFD4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD5')===true&&n.test('\uD837\uDFD5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD6')===true&&n.test('\uD837\uDFD6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD7')===true&&n.test('\uD837\uDFD7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD8')===true&&n.test('\uD837\uDFD8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFD9')===true&&n.test('\uD837\uDFD9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDA')===true&&n.test('\uD837\uDFDA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDB')===true&&n.test('\uD837\uDFDB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDC')===true&&n.test('\uD837\uDFDC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDD')===true&&n.test('\uD837\uDFDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDE')===true&&n.test('\uD837\uDFDE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFDF')===true&&n.test('\uD837\uDFDF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE0')===true&&n.test('\uD837\uDFE0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE1')===true&&n.test('\uD837\uDFE1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE2')===true&&n.test('\uD837\uDFE2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE3')===true&&n.test('\uD837\uDFE3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE4')===true&&n.test('\uD837\uDFE4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE5')===true&&n.test('\uD837\uDFE5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE6')===true&&n.test('\uD837\uDFE6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE7')===true&&n.test('\uD837\uDFE7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE8')===true&&n.test('\uD837\uDFE8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFE9')===true&&n.test('\uD837\uDFE9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFEA')===true&&n.test('\uD837\uDFEA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFEB')===true&&n.test('\uD837\uDFEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFEC')===true&&n.test('\uD837\uDFEC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFED')===true&&n.test('\uD837\uDFED')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFEE')===true&&n.test('\uD837\uDFEE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFEF')===true&&n.test('\uD837\uDFEF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF0')===true&&n.test('\uD837\uDFF0')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF1')===true&&n.test('\uD837\uDFF1')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF2')===true&&n.test('\uD837\uDFF2')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF3')===true&&n.test('\uD837\uDFF3')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF4')===true&&n.test('\uD837\uDFF4')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF5')===true&&n.test('\uD837\uDFF5')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF6')===true&&n.test('\uD837\uDFF6')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF7')===true&&n.test('\uD837\uDFF7')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF8')===true&&n.test('\uD837\uDFF8')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFF9')===true&&n.test('\uD837\uDFF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFA')===true&&n.test('\uD837\uDFFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFB')===true&&n.test('\uD837\uDFFB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFC')===true&&n.test('\uD837\uDFFC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFD')===true&&n.test('\uD837\uDFFD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFE')===true&&n.test('\uD837\uDFFE')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','u'),n=new RegExp('^\\P{Lowercase}$','u');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','ui'),n=new RegExp('^\\P{Lowercase}$','ui');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','v'),n=new RegExp('^\\P{Lowercase}$','v');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lowercase}$','vi'),n=new RegExp('^\\P{Lowercase}$','vi');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','u'),n=new RegExp('^\\P{Lower}$','u');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','ui'),n=new RegExp('^\\P{Lower}$','ui');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','v'),n=new RegExp('^\\P{Lower}$','v');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Lower}$','vi'),n=new RegExp('^\\P{Lower}$','vi');p.test('\uD837\uDFFF')===true&&n.test('\uD837\uDFFF')===false",
    );
}
