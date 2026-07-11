mod testutils;

use aiscript_v1::{
    errors::{AiScriptError, AiScriptRuntimeError},
    values::Value,
};
use testutils::*;

mod core {
    use super::*;

    #[tokio::test]
    async fn range() {
        test("<: Core:range(1, 10)", |res| {
            assert_eq!(
                res,
                arr([
                    num(1),
                    num(2),
                    num(3),
                    num(4),
                    num(5),
                    num(6),
                    num(7),
                    num(8),
                    num(9),
                    num(10),
                ]),
            )
        })
        .await
        .unwrap();

        test("<: Core:range(1, 1)", |res| assert_eq!(res, arr([num(1)])))
            .await
            .unwrap();

        test("<: Core:range(9, 7)", |res| {
            assert_eq!(res, arr([num(9), num(8), num(7)]))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_str() {
        test(r#"<: Core:to_str("abc")"#, |res| {
            assert_eq!(res, str("abc"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(123)"#, |res| assert_eq!(res, str("123")))
            .await
            .unwrap();

        test(r#"<: Core:to_str(true)"#, |res| {
            assert_eq!(res, str("true"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(false)"#, |res| {
            assert_eq!(res, str("false"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(null)"#, |res| {
            assert_eq!(res, str("null"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str({ a: "abc", b: 1234 })"#, |res| {
            assert_eq!(res, str(r#"{ a: "abc", b: 1234 }"#))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str([ true, 123, null ])"#, |res| {
            assert_eq!(res, str("[ true, 123, null ]"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(@( a, b, c ) {})"#, |res| {
            assert_eq!(res, str("@( a, b, c ) { ... }"))
        })
        .await
        .unwrap();

        test(
            r#"
            let arr = []
            arr.push(arr)
            <: Core:to_str(arr)
            "#,
            |res| assert_eq!(res, str("[ ... ]")),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = []
            arr.push({ value: arr })
            <: Core:to_str(arr)
            "#,
            |res| assert_eq!(res, str("[ { value: ... } ]")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn abort() {
        let err = test(r#"Core:abort("hoge")"#, |_| {}).await.unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::User(message)) = err {
            assert_eq!(message, "hoge");
        } else {
            panic!("{err}");
        }
    }
}

mod arr {
    use super::*;

    #[tokio::test]
    async fn create() {
        test("<: Arr:create(0)", |res| assert_eq!(res, arr([])))
            .await
            .unwrap();

        test("<: Arr:create(3)", |res| {
            assert_eq!(res, arr([null(), null(), null()]))
        })
        .await
        .unwrap();

        test("<: Arr:create(3, 1)", |res| {
            assert_eq!(res, arr([num(1), num(1), num(1)]))
        })
        .await
        .unwrap();
    }
}

mod math {
    use super::*;

    #[tokio::test]
    async fn trig() {
        test("<: Math:sin(Math:PI / 2)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();

        test("<: Math:sin(0 - (Math:PI / 2))", |res| {
            assert_eq!(res, num(-1))
        })
        .await
        .unwrap();

        test("<: Math:sin(Math:PI / 4) * Math:cos(Math:PI / 4)", |res| {
            assert!((f64::try_from(res).unwrap() - 0.5).abs() <= f64::EPSILON)
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn abs() {
        test("<: Math:abs(1 - 6)", |res| assert_eq!(res, num(5)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn pow_and_sqrt() {
        test("<: Math:sqrt(3^2 + 4^2)", |res| assert_eq!(res, num(5)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn round() {
        test("<: Math:round(3.14)", |res| assert_eq!(res, num(3)))
            .await
            .unwrap();

        test("<: Math:round(-1.414213)", |res| assert_eq!(res, num(-1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ceil() {
        test("<: Math:ceil(2.71828)", |res| assert_eq!(res, num(3)))
            .await
            .unwrap();

        test("<: Math:ceil(0 - Math:PI)", |res| assert_eq!(res, num(-3)))
            .await
            .unwrap();

        test("<: Math:ceil(1 / Math:Infinity)", |res| {
            assert_eq!(res, num(0))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn floor() {
        test("<: Math:floor(23.14069)", |res| assert_eq!(res, num(23)))
            .await
            .unwrap();

        test("<: Math:floor(Math:Infinity / 0)", |res| {
            assert_eq!(res, num(f64::INFINITY))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn min() {
        test("<: Math:min(2, 3)", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn max() {
        test("<: Math:max(-2, -3)", |res| assert_eq!(res, num(-2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rnd_with_arg() {
        test("<: Math:rnd(1, 1.5)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn gen_rng() {
        // 2つのシード値から1~maxの乱数をn回生成して一致率を見る
        test(
            r#"
            @test(seed1, seed2) {
                let n = 100
                let max = 100000
                let threshold = 0.05
                let random1 = Math:gen_rng(seed1)
                let random2 = Math:gen_rng(seed2)
                var same = 0
                for n {
                    if random1(1, max) == random2(1, max) {
                        same += 1
                    }
                }
                let rate = same / n
                if seed1 == seed2 { rate == 1 }
                else { rate < threshold }
            }
            let seed1 = `{Util:uuid()}`
            let seed2 = `{Date:year()}`
            <: [
                test(seed1, seed1)
                test(seed1, seed2)
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(true)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn gen_rng_number_seed() {
        // 2つのシード値から1~maxの乱数をn回生成して一致率を見る(numがシード値として指定された場合)
        test(
            r#"
            @test(seed1, seed2) {
                let n = 100
                let max = 100000
                let threshold = 0.05
                let random1 = Math:gen_rng(seed1)
                let random2 = Math:gen_rng(seed2)
                var same = 0
                for n {
                    if random1(1, max) == random2(1, max) {
                        same += 1
                    }
                }
                let rate = same / n
                if seed1 == seed2 { rate == 1 }
                else { rate < threshold }
            }
            let seed1 = 3.0
            let seed2 = 3.0000000000000004
            <: [
                test(seed1, seed1)
                test(seed1, seed2)
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(true)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn gen_rng_should_reject_when_null_is_provided_as_a_seed() {
        let err = test("Math:gen_rng(null)", |_| {}).await.unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::InvalidSeed) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn gen_rng_reproducibility() {
        test(
            r#"
            <: ["ai", "👍🏽🍆🌮", 42, -0.12].map(@(seed) {
                [
                    null
                    {algorithm: "chacha20"}
                    {algorithm: "chacha20", chacha20_number_seed_legacy_behavior: true}
                    {algorithm: "rc4"}
                    {algorithm: "rc4_legacy"}
                ].map(@(options) {
                    [[null, null], [0, 10000]].map(@(args) {
                        let rng = if options != null {
                          Math:gen_rng(seed, options)
                        } else {
                          Math:gen_rng(seed)
                        }
                        let result = []
                        for 10 {
                            result.push(rng(args[0], args[1]))
                        }
                        result
                    })
                })
            })
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([
                            arr([
                                arr([
                                    num(0.9522313273998938),
                                    num(0.157311387592687),
                                    num(0.3808835685435835),
                                    num(0.9527526716026183),
                                    num(0.6877129153536835),
                                    num(0.7029341236519296),
                                    num(0.6563772673880722),
                                    num(0.9290612890051076),
                                    num(0.7671812917264167),
                                    num(0.7613553489274978),
                                ]),
                                arr([
                                    num(8892),
                                    num(9160),
                                    num(2577),
                                    num(7839),
                                    num(9593),
                                    num(6240),
                                    num(4822),
                                    num(9815),
                                    num(1754),
                                    num(2078),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.9522313273998938),
                                    num(0.157311387592687),
                                    num(0.3808835685435835),
                                    num(0.9527526716026183),
                                    num(0.6877129153536835),
                                    num(0.7029341236519296),
                                    num(0.6563772673880722),
                                    num(0.9290612890051076),
                                    num(0.7671812917264167),
                                    num(0.7613553489274978),
                                ]),
                                arr([
                                    num(8892),
                                    num(9160),
                                    num(2577),
                                    num(7839),
                                    num(9593),
                                    num(6240),
                                    num(4822),
                                    num(9815),
                                    num(1754),
                                    num(2078),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.9522313273998938),
                                    num(0.157311387592687),
                                    num(0.3808835685435835),
                                    num(0.9527526716026183),
                                    num(0.6877129153536835),
                                    num(0.7029341236519296),
                                    num(0.6563772673880722),
                                    num(0.9290612890051076),
                                    num(0.7671812917264167),
                                    num(0.7613553489274978),
                                ]),
                                arr([
                                    num(8892),
                                    num(9160),
                                    num(2577),
                                    num(7839),
                                    num(9593),
                                    num(6240),
                                    num(4822),
                                    num(9815),
                                    num(1754),
                                    num(2078),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.3918487679914174),
                                    num(0.49612542958789424),
                                    num(0.7684297416081407),
                                    num(0.21775697119876608),
                                    num(0.3058367971077552),
                                    num(0.2767969751830176),
                                    num(0.8547817961904522),
                                    num(0.6042791647837444),
                                    num(0.7811502995575069),
                                    num(0.57021387570539),
                                ]),
                                arr([
                                    num(4316),
                                    num(6880),
                                    num(5132),
                                    num(4143),
                                    num(8114),
                                    num(132),
                                    num(949),
                                    num(3567),
                                    num(303),
                                    num(8047),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.43852999322126746),
                                    num(0.8817787644203001),
                                    num(0.4995854530677725),
                                    num(0.7178449229747838),
                                    num(0.7430381189910698),
                                    num(0.32146160279166847),
                                    num(0.6412789985363597),
                                    num(0.8547970547462124),
                                    num(0.3987127958337033),
                                    num(0.8605570778668624),
                                ]),
                                arr([
                                    num(4385),
                                    num(8818),
                                    num(4996),
                                    num(7179),
                                    num(7431),
                                    num(3214),
                                    num(6413),
                                    num(8548),
                                    num(3987),
                                    num(8606),
                                ]),
                            ]),
                        ]),
                        arr([
                            arr([
                                arr([
                                    num(0.01658515362611132),
                                    num(0.40211525808984927),
                                    num(0.5416933861007267),
                                    num(0.9652372121672298),
                                    num(0.9943497429151748),
                                    num(0.03290852032834708),
                                    num(0.3075534222881005),
                                    num(0.9323756828284806),
                                    num(0.7118891994071928),
                                    num(0.6047536199660608),
                                ]),
                                arr([
                                    num(5909),
                                    num(4027),
                                    num(8877),
                                    num(5337),
                                    num(1693),
                                    num(8738),
                                    num(2301),
                                    num(5095),
                                    num(6822),
                                    num(9069),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.01658515362611132),
                                    num(0.40211525808984927),
                                    num(0.5416933861007267),
                                    num(0.9652372121672298),
                                    num(0.9943497429151748),
                                    num(0.03290852032834708),
                                    num(0.3075534222881005),
                                    num(0.9323756828284806),
                                    num(0.7118891994071928),
                                    num(0.6047536199660608),
                                ]),
                                arr([
                                    num(5909),
                                    num(4027),
                                    num(8877),
                                    num(5337),
                                    num(1693),
                                    num(8738),
                                    num(2301),
                                    num(5095),
                                    num(6822),
                                    num(9069),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.01658515362611132),
                                    num(0.40211525808984927),
                                    num(0.5416933861007267),
                                    num(0.9652372121672298),
                                    num(0.9943497429151748),
                                    num(0.03290852032834708),
                                    num(0.3075534222881005),
                                    num(0.9323756828284806),
                                    num(0.7118891994071928),
                                    num(0.6047536199660608),
                                ]),
                                arr([
                                    num(5909),
                                    num(4027),
                                    num(8877),
                                    num(5337),
                                    num(1693),
                                    num(8738),
                                    num(2301),
                                    num(5095),
                                    num(6822),
                                    num(9069),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.785901577584311),
                                    num(0.8581770053926902),
                                    num(0.39279265728812507),
                                    num(0.42995532689596705),
                                    num(0.8237803317852699),
                                    num(0.40854037448894154),
                                    num(0.9018411110779502),
                                    num(0.1246844544394192),
                                    num(0.2263238136007084),
                                    num(0.0661723643559127),
                                ]),
                                arr([
                                    num(7408),
                                    num(8550),
                                    num(3126),
                                    num(5583),
                                    num(7999),
                                    num(5357),
                                    num(4170),
                                    num(9091),
                                    num(3225),
                                    num(5684),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.7634826614159379),
                                    num(0.9814352981017643),
                                    num(0.7122064384307634),
                                    num(0.19849161859342623),
                                    num(0.43077588526244975),
                                    num(0.8899309964673247),
                                    num(0.587528000034823),
                                    num(0.06201019022786278),
                                    num(0.9184540672201553),
                                    num(0.3591342686183971),
                                ]),
                                arr([
                                    num(7635),
                                    num(9815),
                                    num(7122),
                                    num(1985),
                                    num(4308),
                                    num(8900),
                                    num(5875),
                                    num(620),
                                    num(9185),
                                    num(3591),
                                ]),
                            ]),
                        ]),
                        arr([
                            arr([
                                arr([
                                    num(0.3900426548095348),
                                    num(0.08515097961982344),
                                    num(0.32710240729179996),
                                    num(0.19291286032499458),
                                    num(0.6584090738680834),
                                    num(0.634180356163491),
                                    num(0.8304177952365436),
                                    num(0.33722997819744294),
                                    num(0.17470509339869209),
                                    num(0.9948637088923301),
                                ]),
                                arr([
                                    num(4696),
                                    num(8406),
                                    num(6188),
                                    num(3013),
                                    num(5012),
                                    num(6319),
                                    num(4812),
                                    num(3032),
                                    num(4073),
                                    num(9055),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.3900426548095348),
                                    num(0.08515097961982344),
                                    num(0.32710240729179996),
                                    num(0.19291286032499458),
                                    num(0.6584090738680834),
                                    num(0.634180356163491),
                                    num(0.8304177952365436),
                                    num(0.33722997819744294),
                                    num(0.17470509339869209),
                                    num(0.9948637088923301),
                                ]),
                                arr([
                                    num(4696),
                                    num(8406),
                                    num(6188),
                                    num(3013),
                                    num(5012),
                                    num(6319),
                                    num(4812),
                                    num(3032),
                                    num(4073),
                                    num(9055),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.3582586927375501),
                                    num(0.019206453806487965),
                                    num(0.6044005114943116),
                                    num(0.06933935753438714),
                                    num(0.6064448575752989),
                                    num(0.1713002060360721),
                                    num(0.8591720709605983),
                                    num(0.7957404008540241),
                                    num(0.5609916707024384),
                                    num(0.9070083696790407),
                                ]),
                                arr([
                                    num(7732),
                                    num(9438),
                                    num(8102),
                                    num(3672),
                                    num(1136),
                                    num(5891),
                                    num(1856),
                                    num(9935),
                                    num(1407),
                                    num(9134),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.8695755235155893),
                                    num(0.5645109226444853),
                                    num(0.9910554516954708),
                                    num(0.14484532954921459),
                                    num(0.40017802949770215),
                                    num(0.8740010123230115),
                                    num(0.6320973510631253),
                                    num(0.2377727506114171),
                                    num(0.7435184728391848),
                                    num(0.5954728937200902),
                                ]),
                                arr([
                                    num(6329),
                                    num(7543),
                                    num(9248),
                                    num(6243),
                                    num(7807),
                                    num(6426),
                                    num(9560),
                                    num(2373),
                                    num(4429),
                                    num(1103),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.00701751618236155),
                                    num(0.17185490054868188),
                                    num(0.967001069269818),
                                    num(0.4077816952668805),
                                    num(0.922687842759339),
                                    num(0.7869285107049383),
                                    num(0.873204273355561),
                                    num(0.6298147295125774),
                                    num(0.23825186137402535),
                                    num(0.34274772768400263),
                                ]),
                                arr([
                                    num(70),
                                    num(1718),
                                    num(9670),
                                    num(4078),
                                    num(9227),
                                    num(7870),
                                    num(8732),
                                    num(6298),
                                    num(2382),
                                    num(3427),
                                ]),
                            ]),
                        ]),
                        arr([
                            arr([
                                arr([
                                    num(0.20789063739852662),
                                    num(0.5515801296852669),
                                    num(0.8602687843260233),
                                    num(0.5078228623716351),
                                    num(0.9279083097419194),
                                    num(0.9658646731821696),
                                    num(0.3115872919847804),
                                    num(0.09283929980057701),
                                    num(0.30940603534094757),
                                    num(0.46543072210339276),
                                ]),
                                arr([
                                    num(7276),
                                    num(2097),
                                    num(3604),
                                    num(2377),
                                    num(3350),
                                    num(7371),
                                    num(9421),
                                    num(1707),
                                    num(43),
                                    num(461),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.20789063739852662),
                                    num(0.5515801296852669),
                                    num(0.8602687843260233),
                                    num(0.5078228623716351),
                                    num(0.9279083097419194),
                                    num(0.9658646731821696),
                                    num(0.3115872919847804),
                                    num(0.09283929980057701),
                                    num(0.30940603534094757),
                                    num(0.46543072210339276),
                                ]),
                                arr([
                                    num(7276),
                                    num(2097),
                                    num(3604),
                                    num(2377),
                                    num(3350),
                                    num(7371),
                                    num(9421),
                                    num(1707),
                                    num(43),
                                    num(461),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.7030147638265518),
                                    num(0.8910625923471456),
                                    num(0.49960942726392166),
                                    num(0.5028878135621331),
                                    num(0.30439784844478524),
                                    num(0.4507681928982245),
                                    num(0.3815911306991666),
                                    num(0.5745189357866595),
                                    num(0.10187935040528222),
                                    num(0.7601333172603988),
                                ]),
                                arr([
                                    num(8064),
                                    num(8911),
                                    num(9324),
                                    num(3106),
                                    num(4535),
                                    num(671),
                                    num(6793),
                                    num(7731),
                                    num(706),
                                    num(285),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.12804382060971414),
                                    num(0.1348380245197581),
                                    num(0.600869126584787),
                                    num(0.7363981406253993),
                                    num(0.6075140094218262),
                                    num(0.693848055489433),
                                    num(0.8968020135686416),
                                    num(0.33188423574857273),
                                    num(0.29629555610425584),
                                    num(0.29239040461391297),
                                ]),
                                arr([
                                    num(4013),
                                    num(8495),
                                    num(9544),
                                    num(2038),
                                    num(9642),
                                    num(1907),
                                    num(8578),
                                    num(1638),
                                    num(428),
                                    num(1488),
                                ]),
                            ]),
                            arr([
                                arr([
                                    num(0.711898436183017),
                                    num(0.7412539463637078),
                                    num(0.13643077524750266),
                                    num(0.8226574810275145),
                                    num(0.5185058185204597),
                                    num(0.525804126337512),
                                    num(0.6277132421410834),
                                    num(0.5816253487951227),
                                    num(0.9622246210896411),
                                    num(0.026768397880120472),
                                ]),
                                arr([
                                    num(7119),
                                    num(7413),
                                    num(1364),
                                    num(8227),
                                    num(5185),
                                    num(5258),
                                    num(6277),
                                    num(5816),
                                    num(9623),
                                    num(267),
                                ]),
                            ]),
                        ]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }
}

mod obj {
    use super::*;

    #[tokio::test]
    async fn keys() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3, }

            <: Obj:keys(o)
            "#,
            |res| assert_eq!(res, arr([str("a"), str("b"), str("c")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn vals() {
        test(
            r#"
            let o = { _nul: null, _num: 24, _str: 'hoge', _arr: [], _obj: {}, }

            <: Obj:vals(o)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        null(),
                        num(24),
                        str("hoge"),
                        arr([]),
                        obj([] as [(String, Value); 0]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn kvs() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3, }

            <: Obj:kvs(o)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([str("a"), num(1)]),
                        arr([str("b"), num(2)]),
                        arr([str("c"), num(3)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn merge() {
        test(
            r#"
            let o1 = { a: 1, b: 2 }
            let o2 = { b: 3, c: 4 }

            <: Obj:merge(o1, o2)
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(3)), ("c", num(4))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pick() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3 }

            <: Obj:pick(o, ['b', 'd'])
            "#,
            |res| assert_eq!(res, obj([("b", num(2)), ("d", null())])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_kvs() {
        test(
            r#"
            let kvs = [['a', 1], ['b', 2], ['c', 3]]

            <: Obj:from_kvs(kvs)
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }
}

mod str {
    use super::*;

    #[tokio::test]
    async fn lf() {
        test(
            r#"
            <: Str:lf
            "#,
            |res| assert_eq!(res, str("\n")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_codepoint() {
        test(
            r#"
            <: Str:from_codepoint(65)
            "#,
            |res| assert_eq!(res, str("A")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_unicode_codepoints() {
        test(
            r#"
            <: Str:from_unicode_codepoints([171581, 128073, 127999, 128104, 8205, 128102])
            "#,
            |res| assert_eq!(res, str("𩸽👉🏿👨‍👦")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_utf8_bytes() {
        test(
            r#"
            <: Str:from_utf8_bytes([240, 169, 184, 189, 240, 159, 145, 137, 240, 159, 143, 191, 240, 159, 145, 168, 226, 128, 141, 240, 159, 145, 166])
            "#,
            |res| assert_eq!(res, str("𩸽👉🏿👨‍👦")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn charcode_at() {
        test(
            r#"
            <: "aiscript".split().map(@(x, _) { x.charcode_at(0) })
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(97),
                        num(105),
                        num(115),
                        num(99),
                        num(114),
                        num(105),
                        num(112),
                        num(116),
                    ]),
                )
            },
        )
        .await
        .unwrap();

        test(
            r#"
            <: "".charcode_at(0)
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }
}

mod uri {
    use super::*;

    #[tokio::test]
    async fn encode_full() {
        test(
            r#"
            <: Uri:encode_full("https://example.com/?q=あいちゃん")
            "#,
            |res| {
                assert_eq!(
                    res,
                    str("https://example.com/?q=%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93"),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn encode_component() {
        test(
            r#"
            <: Uri:encode_component("https://example.com/?q=あいちゃん")
            "#,
            |res| assert_eq!(res, str("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn decode_full() {
        test(
            r#"
            <: Uri:decode_full("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")
            "#,
            |res| assert_eq!(res, str("https%3A%2F%2Fexample.com%2F%3Fq%3Dあいちゃん")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn decode_component() {
        test(
            r#"
            <: Uri:decode_component("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")
            "#,
            |res| assert_eq!(res, str("https://example.com/?q=あいちゃん")),
        )
        .await
        .unwrap();
    }
}

mod error {
    use super::*;

    #[tokio::test]
    async fn create() {
        test(
            r#"
            <: Error:create('ai', {chan: 'kawaii'})
            "#,
            |res| assert_eq!(res, error("ai", Some(obj([("chan", str("kawaii"))])))),
        )
        .await
        .unwrap();
    }
}

mod json {
    use super::*;

    #[tokio::test]
    async fn stringify_fn() {
        test(
            r#"
            <: Json:stringify(@(){})
            "#,
            |res| assert_eq!(res, str(r#""<function>""#)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn parsable() {
        test(
            r#"
            <: [
                Json:parsable('null')
                Json:stringify(Json:parse('null'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("null")])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('"hoge"')
                Json:stringify(Json:parse('"hoge"'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str(r#""hoge""#)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('[]')
                Json:stringify(Json:parse('[]'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("[]")])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('{}')
                Json:stringify(Json:parse('{}'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("{}")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn unparsable() {
        test(
            r#"
            <: [
                Json:parsable('')
                Json:parse('')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('hoge')
                Json:parse('hoge')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('[')
                Json:parse('[')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();
    }
}

mod date {
    use chrono::{Datelike, Local, NaiveDate, TimeZone, Timelike};

    use super::*;

    #[tokio::test]
    async fn year() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:year(0), Date:year({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(Local.timestamp_millis_opt(0).unwrap().year()),
                        num(2024),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn month() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:month(0), Date:month({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().month()), num(1)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn day() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:day(0), Date:day({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().day()), num(2)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn hour() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:hour(0), Date:hour({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().hour()), num(3)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn minute() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:minute(0), Date:minute({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().minute()), num(4)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn second() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:second(0), Date:second({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().second()), num(5)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn millisecond() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:millisecond(0), Date:millisecond({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(
                            (Local.timestamp_millis_opt(0).unwrap().timestamp_millis() % 1000)
                                as f64
                        ),
                        num(6),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                let res = <Vec<Value>>::try_from(res).unwrap();
                assert_eq!(res[0], res[1]);
                let s1 = String::try_from(res[2].clone()).unwrap();
                regex::Regex::new(
                    r"(?x)
                    ^[0-9]{4,4}-[0-9]{2,2}-[0-9]{2,2}T
                    [0-9]{2,2}:[0-9]{2,2}:[0-9]{2,2}\.[0-9]{3,3}
                    (Z|[-+][0-9]{2,2}:[0-9]{2,2})$
                    ",
                )
                .unwrap()
                .captures(&s1)
                .unwrap();
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_utc() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, 0)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-11T16:47:46.021Z")
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_09_00() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, 9*60)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-12T01:47:46.021+09:00")
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_05_18() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, -5*60-18)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-11T11:29:46.021-05:18"),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn parse() {
        test(
            r#"
            <: [
                '01 Jan 1970 00:00:00 GMT'
                '1970-01-01'
                '1970-01-01T00:00:00.000Z'
                '1970-01-01T00:00:00.000+00:00'
                'hoge'
            ].map(Date:parse)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([num(0), num(0), num(0), num(0), error("not_date", None)]),
                )
            },
        )
        .await
        .unwrap();
    }
}
