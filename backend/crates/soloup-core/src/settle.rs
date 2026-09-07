//! §3.3 / §3.5 逐日结算算法（离散，唯一权威）纯函数。
//! 对应 TS `@soloup/core/settle.ts`。

/// 双账户：C 永不衰减（§3.2）；V 随遗忘与结晶流失。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accounts {
    pub c: f64,
    pub v: f64,
}

pub const ZERO_ACCOUNTS: Accounts = Accounts { c: 0.0, v: 0.0 };

#[derive(Debug, Clone, Copy)]
pub struct SettleParams {
    /// 结晶率 c（附录 A）
    pub c: f64,
    /// 遗忘率 f（附录 A）
    pub f: f64,
}

/// d = c + f（§3.4，V 的每日总衰减比例）。
pub fn total_decay(p: SettleParams) -> f64 {
    p.c + p.f
}

/// §3.3 settleDay —— 打卡加 V（每天最多 +1）→ 结晶 C += c·V → 衰减 V *= (1−c−f)。
pub fn settle_day(acc: Accounts, p: SettleParams, has_checkin: bool) -> Accounts {
    let v0 = acc.v + if has_checkin { 1.0 } else { 0.0 };
    let d = total_decay(p);
    Accounts {
        c: acc.c + p.c * v0,
        v: v0 * (1.0 - d),
    }
}

/// §3.5 空窗闭合公式（n 天均无打卡，等价于逐日）。
/// `days` 须为非负有限数（§3.5）。
pub fn apply_gap_closed_form(acc: Accounts, p: SettleParams, days: f64) -> Accounts {
    if !days.is_finite() || days < 0.0 {
        panic!("applyGapClosedForm: days 须为非负有限数，收到 {days}");
    }
    let d = total_decay(p);
    let factor = (1.0 - d).powf(days);
    if d <= 0.0 {
        // c/f 参数保护：d=0 时无流失，空窗无变化。
        return acc;
    }
    Accounts {
        c: acc.c + (p.c / d) * acc.v * (1.0 - factor),
        v: acc.v * factor,
    }
}

/// 逐日重放任意打卡序列。返回 (最终账户, per_day[i] = 第 i+1 天结算后)。
pub fn replay_days(
    initial: Accounts,
    p: SettleParams,
    checkins: &[bool],
) -> (Accounts, Vec<Accounts>) {
    let mut cur = initial;
    let mut per_day = Vec::with_capacity(checkins.len());
    for &ok in checkins {
        cur = settle_day(cur, p, ok);
        per_day.push(cur);
    }
    (cur, per_day)
}
