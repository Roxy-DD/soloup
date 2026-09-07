//! store 层结构化错误（对应 TS `@soloup/store/errors.ts`）。

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreErrorCode {
    NotFound,
    Validation,
    Constraint,
    Fk,
    Linked,
    HasChildren,
    HasHistory,
    Cycle,
    NewerSchema,
    CorruptSettings,
}

impl StoreErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StoreErrorCode::NotFound => "ERR_NOT_FOUND",
            StoreErrorCode::Validation => "ERR_VALIDATION",
            StoreErrorCode::Constraint => "ERR_CONSTRAINT",
            StoreErrorCode::Fk => "ERR_FK",
            StoreErrorCode::Linked => "ERR_LINKED",
            StoreErrorCode::HasChildren => "ERR_HAS_CHILDREN",
            StoreErrorCode::HasHistory => "ERR_HAS_HISTORY",
            StoreErrorCode::Cycle => "ERR_CYCLE",
            StoreErrorCode::NewerSchema => "ERR_NEWER_SCHEMA",
            StoreErrorCode::CorruptSettings => "ERR_SETTINGS_CORRUPT",
        }
    }
}

#[derive(Debug)]
pub struct StoreError {
    pub code: StoreErrorCode,
    pub message: String,
}

impl StoreError {
    pub fn new(code: StoreErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StoreErrorCode::NotFound, message)
    }
    pub fn constraint(code: StoreErrorCode, message: impl Into<String>) -> Self {
        Self::new(code, message)
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(ferr, msg) = &e {
            if ferr.code == rusqlite::ErrorCode::ConstraintViolation {
                return StoreError::constraint(
                    StoreErrorCode::Fk,
                    format!("违反外键/唯一约束：{}", msg.as_deref().unwrap_or("")),
                );
            }
        }
        StoreError::constraint(StoreErrorCode::Constraint, e.to_string())
    }
}

impl From<rusqlite::ffi::Error> for StoreError {
    fn from(e: rusqlite::ffi::Error) -> Self {
        StoreError::constraint(StoreErrorCode::Constraint, format!("sqlite ffi error: {e}"))
    }
}
