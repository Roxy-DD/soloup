//! §3.10 黄金数值表 —— 普通难度、连续每日打卡的权威期望值。
//! 对应 TS `@soloup/core/goldens.ts`。表内数值为 3 位小数展示位；
//! 测试策略：逐日重放结果 round(3) 与表一致。

use crate::SkillCategory;

pub struct GoldenRow {
    pub day: i32,
    pub c: f64,
    pub v: f64,
    pub e: f64,
    pub level: f64,
}

pub fn golden_continuous(cat: SkillCategory) -> &'static [GoldenRow] {
    match cat {
        SkillCategory::Physical => &PHYSICAL,
        SkillCategory::Cognitive => &COGNITIVE,
        SkillCategory::Knowledge => &KNOWLEDGE,
    }
}

const PHYSICAL: &[GoldenRow] = &[
    GoldenRow { day: 0, c: 0.0, v: 0.0, e: 0.0, level: 0.0 },
    GoldenRow { day: 1, c: 0.015, v: 0.981, e: 1.0, level: 0.21 },
    GoldenRow { day: 7, c: 0.404, v: 6.488, e: 6.9, level: 1.43 },
    GoldenRow { day: 30, c: 5.848, v: 22.592, e: 28.4, level: 5.75 },
    GoldenRow { day: 100, c: 44.172, v: 44.049, e: 88.2, level: 16.79 },
    GoldenRow { day: 365, c: 247.433, v: 51.585, e: 299.0, level: 46.36 },
    GoldenRow { day: 1095, c: 823.712, v: 51.632, e: 875.3, level: 83.86 },
    GoldenRow { day: 1825, c: 1400.028, v: 51.632, e: 1451.7, level: 95.14 },
];

const COGNITIVE: &[GoldenRow] = &[
    GoldenRow { day: 0, c: 0.0, v: 0.0, e: 0.0, level: 0.0 },
    GoldenRow { day: 1, c: 0.01, v: 0.982, e: 1.0, level: 0.13 },
    GoldenRow { day: 7, c: 0.27, v: 6.514, e: 6.8, level: 0.87 },
    GoldenRow { day: 30, c: 3.934, v: 22.919, e: 26.9, level: 3.55 },
    GoldenRow { day: 100, c: 30.175, v: 45.684, e: 75.9, level: 10.84 },
    GoldenRow { day: 365, c: 172.509, v: 54.484, e: 227.0, level: 37.97 },
    GoldenRow { day: 1095, c: 578.025, v: 54.556, e: 632.6, level: 90.17 },
    GoldenRow { day: 1825, c: 983.58, v: 54.556, e: 1038.1, level: 99.07 },
];

const KNOWLEDGE: &[GoldenRow] = &[
    GoldenRow { day: 0, c: 0.0, v: 0.0, e: 0.0, level: 0.0 },
    GoldenRow { day: 1, c: 0.006, v: 0.98, e: 1.0, level: 0.15 },
    GoldenRow { day: 7, c: 0.161, v: 6.462, e: 6.6, level: 1.0 },
    GoldenRow { day: 30, c: 2.319, v: 22.271, e: 24.6, level: 3.96 },
    GoldenRow { day: 100, c: 17.25, v: 42.502, e: 59.8, level: 10.8 },
    GoldenRow { day: 365, c: 94.809, v: 48.969, e: 143.8, level: 32.22 },
    GoldenRow { day: 1095, c: 313.8, v: 49.0, e: 362.8, level: 84.43 },
    GoldenRow { day: 1825, c: 532.8, v: 49.0, e: 581.8, level: 98.03 },
];

/// §3.10 断更行为（cognitive，打卡 100 天后断更 60 天）。
pub struct GoldenGap {
    pub day100_c: f64,
    pub day100_v: f64,
    pub day100_e: f64,
    pub day100_level: f64,
    pub day160_c: f64,
    pub day160_v: f64,
    pub day160_e: f64,
    pub day160_level: f64,
}

pub const GOLDEN_COGNITIVE_GAP: GoldenGap = GoldenGap {
    day100_c: 30.175,
    day100_v: 45.684,
    day100_e: 75.9,
    day100_level: 10.84,
    day160_c: 47.021,
    day160_v: 15.362,
    day160_e: 62.4,
    day160_level: 8.73,
};
