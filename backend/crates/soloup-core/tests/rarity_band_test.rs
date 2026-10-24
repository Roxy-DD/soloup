//! 稀有度分档的边界测试。
//!
//! 这条规则的存在理由不是「好看」，而是**单调**：卡墙按 `points` 升序摆，
//! 稀有度若不随分值单调，就会出现金卡夹在紫卡中间（真实库里的「完稿 150」）。

use soloup_core::schema::{DbEnum, Rarity};

fn band(points: i64) -> &'static str {
    Rarity::from_points(points).as_str()
}

#[test]
fn bands_are_monotonic_across_the_real_point_range() {
    // 真实成就表 22 条的分值：10,15,20,20,20 | 30,40,45,50,60,60 | 100,110,120,130,150,160,180 | 200,220,280,300
    let points: [i64; 22] = [
        10, 15, 20, 20, 20, 30, 40, 45, 50, 60, 60, 100, 110, 120, 130, 150, 160, 180, 200, 220,
        280, 300,
    ];
    let rank = |r: &str| match r {
        "common" => 0,
        "rare" => 1,
        "epic" => 2,
        "legendary" => 3,
        other => panic!("未知稀有度 {other}"),
    };
    let ranks: Vec<i32> = points.iter().map(|p| rank(band(*p))).collect();
    assert!(
        ranks.windows(2).all(|w| w[0] <= w[1]),
        "分值升序时稀有度不得下降，实际 {ranks:?}"
    );
}

#[test]
fn band_boundaries() {
    assert_eq!(band(0), "common");
    assert_eq!(band(20), "common");
    assert_eq!(band(21), "rare");
    assert_eq!(band(60), "rare");
    assert_eq!(band(61), "epic");
    assert_eq!(band(199), "epic");
    assert_eq!(band(200), "legendary");
    assert_eq!(band(100_000), "legendary");
}

#[test]
fn unassigned_rarity_is_rejected_so_the_band_is_the_single_source() {
    // 分档之外的取值不该被当成稀有度（防止有人手写 'mythic' 绕过规则）
    assert!(Rarity::parse("mythic").is_none());
}
