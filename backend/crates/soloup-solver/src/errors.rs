//! solver 服务层错误（对应 TS `@soloup/solver/errors.ts`）。

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverErrorCode {
    Validation,
    NotFound,
    NonLeaf,
    Archived,
    FutureDate,
    PastEdit,
    CatchUpTooLong,
}

impl SolverErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SolverErrorCode::Validation => "ERR_VALIDATION",
            SolverErrorCode::NotFound => "ERR_NOT_FOUND",
            SolverErrorCode::NonLeaf => "ERR_NON_LEAF",
            SolverErrorCode::Archived => "ERR_SKILL_ARCHIVED",
            SolverErrorCode::FutureDate => "ERR_FUTURE_DATE",
            SolverErrorCode::PastEdit => "ERR_HISTORY_EDIT",
            SolverErrorCode::CatchUpTooLong => "ERR_CATCHUP_TOO_LONG",
        }
    }
}

#[derive(Debug)]
pub struct SolverError {
    pub code: SolverErrorCode,
    pub message: String,
}

impl SolverError {
    pub fn new(code: SolverErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for SolverError {}

impl From<soloup_store::StoreError> for SolverError {
    fn from(e: soloup_store::StoreError) -> Self {
        use soloup_store::StoreErrorCode;
        let code = match e.code {
            StoreErrorCode::NotFound => SolverErrorCode::NotFound,
            _ => SolverErrorCode::Validation,
        };
        SolverError::new(code, e.message)
    }
}

impl From<rusqlite::Error> for SolverError {
    fn from(e: rusqlite::Error) -> Self {
        SolverError::new(
            SolverErrorCode::Validation,
            format!("数据库错误：{e}"),
        )
    }
}

/// 防呆断言：condition 不满足即抛服务层错误。
pub fn solver_assert(condition: bool, code: SolverErrorCode, message: impl Into<String>) -> Result<(), SolverError> {
    if condition {
        Ok(())
    } else {
        Err(SolverError::new(code, message))
    }
}
