//! 纯结算引擎（对应 TS `ledger.ts`）：给定账户/参数/打卡分布，推进到目标日期。
//! 逐日与闭式空窗等价、参数跨段边界强制刷新。

use soloup_core::dates::{add_days, compare_iso, diff_days};
use soloup_core::params::CATCH_UP_DAY_CAP;
use soloup_core::settle::{apply_gap_closed_form, settle_day, Accounts, SettleParams};

use crate::errors::{SolverError, SolverErrorCode};

/// 从 from（含）增量推进到 to（含）：打卡日 settleDay、无打卡累积后闭式闭合。
pub fn settle_range(
    accounts: Accounts,
    from: &str,
    to: &str,
    is_checkin: &dyn Fn(&str) -> bool,
    p: SettleParams,
) -> Result<Accounts, SolverError> {
    let days = diff_days(from, to) + 1;
    if days <= 0 {
        return Ok(accounts);
    }
    if days > CATCH_UP_DAY_CAP {
        return Err(SolverError::new(
            SolverErrorCode::CatchUpTooLong,
            format!("单次结算跨度 {days} 天超出防呆上限 {CATCH_UP_DAY_CAP}（§3.5）"),
        ));
    }
    let mut cur = accounts;
    let mut gap: i64 = 0;
    for i in 0..days {
        let d = add_days(from, i);
        if is_checkin(&d) {
            if gap > 0 {
                cur = apply_gap_closed_form(cur, p, gap as f64);
                gap = 0;
            }
            cur = settle_day(cur, p, true);
        } else {
            gap += 1;
        }
    }
    if gap > 0 {
        cur = apply_gap_closed_form(cur, p, gap as f64);
    }
    Ok(cur)
}

/// 参数分段（§C.6）。`from` 起始日（含），params 为该段生效结算参数。
#[derive(Debug, Clone)]
pub struct ParamSegment {
    pub from: String,
    pub accounts: SettleParams,
}

/// 全量重放选项。
pub struct ReplayOptions<'a> {
    pub initial: Accounts,
    pub created_at: &'a str,
    pub to_date: &'a str,
    pub segments: &'a [ParamSegment],
    pub is_checkin: &'a dyn Fn(&str) -> bool,
}

/// 全量重放（§3.5）：从创建日用「当时的参数」重演。参数段切换日强制刷新累积空窗。
pub fn replay_full(opts: ReplayOptions) -> Result<Accounts, SolverError> {
    let mut sorted: Vec<&ParamSegment> = opts.segments.iter().collect();
    sorted.sort_by(|a, b| compare_iso(&a.from, &b.from));
    if sorted.is_empty() || compare_iso(&sorted[0].from, opts.created_at) == std::cmp::Ordering::Greater {
        return Err(SolverError::new(
            SolverErrorCode::Validation,
            "参数分段为空或首段晚于技能创建日，无法重放",
        ));
    }
    let days = diff_days(opts.created_at, opts.to_date) + 1;
    if days <= 0 {
        return Ok(opts.initial);
    }
    if days > CATCH_UP_DAY_CAP {
        return Err(SolverError::new(
            SolverErrorCode::CatchUpTooLong,
            format!("全量重放跨度 {days} 天超出防呆上限 {CATCH_UP_DAY_CAP}"),
        ));
    }

    let mut si: usize = 0;
    while si + 1 < sorted.len()
        && compare_iso(&sorted[si + 1].from, opts.created_at) != std::cmp::Ordering::Greater
    {
        si += 1;
    }

    let mut cur = opts.initial;
    let mut gap: i64 = 0;
    let flush = |cur: &mut Accounts, gap: &mut i64, seg: &ParamSegment| {
        if *gap > 0 {
            *cur = apply_gap_closed_form(*cur, seg.accounts, *gap as f64);
            *gap = 0;
        }
    };

    for i in 0..days {
        let d = add_days(opts.created_at, i);
        while si + 1 < sorted.len() && compare_iso(&sorted[si + 1].from, &d) != std::cmp::Ordering::Greater {
            si += 1;
        }
        let seg = sorted[si];
        if (opts.is_checkin)(&d) {
            flush(&mut cur, &mut gap, seg);
            cur = settle_day(cur, seg.accounts, true);
        } else {
            gap += 1;
        }
        // 次日将进入新参数段 → 本段空窗必须在此收口（不能跨段闭式）
        if si + 1 < sorted.len() && compare_iso(&d, &add_days(&sorted[si + 1].from, -1)) == std::cmp::Ordering::Equal {
            flush(&mut cur, &mut gap, seg);
        }
    }
    flush(&mut cur, &mut gap, sorted[si]);
    Ok(cur)
}
