//! ISO 日期工具（对应 `@soloup/core/dates`）—— §3.5 本地自然日 YYYY-MM-DD。
//! 纯日历运算，内部用 NaiveDate 做日历差，避免 DST 误差。

use chrono::{Datelike, Local, NaiveDate};

pub type IsoDate = String;

fn pad2(n: u32) -> String {
    if n < 10 {
        format!("0{n}")
    } else {
        n.to_string()
    }
}

/// 解析并校验日历日期；非法返回 None（用回环验证真实存在，如 2024-02-30 被拒）。
pub fn parse_iso_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    let d: u32 = parts[2].parse().ok()?;
    NaiveDate::from_ymd_opt(y, m, d)
}

pub fn is_valid_iso_date(s: &str) -> bool {
    parse_iso_date(s).is_some()
}

/// 今天的本地自然日。
pub fn today_iso() -> IsoDate {
    to_iso_date(Local::now().date_naive())
}

pub fn to_iso_date(date: NaiveDate) -> IsoDate {
    format!("{}-{}-{}", date.year(), pad2(date.month()), pad2(date.day()))
}

/// 加 n 天（n 可为负）。输入非法则 panic（防呆，§3.5）。
pub fn add_days(iso: &str, n: i64) -> IsoDate {
    let c = parse_iso_date(iso).unwrap_or_else(|| panic!("add_days: 非法日期 {iso}"));
    to_iso_date(c + chrono::Duration::days(n))
}

/// 日历天数差（to - from）。
pub fn diff_days(from: &str, to: &str) -> i64 {
    let a = parse_iso_date(from).unwrap_or_else(|| panic!("diff_days: 非法日期 {from}"));
    let b = parse_iso_date(to).unwrap_or_else(|| panic!("diff_days: 非法日期 {to}"));
    (b - a).num_days()
}

/// 生成 [from, to] 闭区间逐日序列（含两端）；from > to 时返回空。
pub fn each_day(from: &str, to: &str) -> Vec<IsoDate> {
    let days = diff_days(from, to);
    if days < 0 {
        return vec![];
    }
    (0..=days).map(|i| add_days(from, i)).collect()
}

/// 字典序比较即日历序（YYYY-MM-DD 固定宽度）。
pub fn compare_iso(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

pub fn is_before_or_equal(from: &str, to: &str) -> bool {
    compare_iso(from, to) != std::cmp::Ordering::Greater
}
