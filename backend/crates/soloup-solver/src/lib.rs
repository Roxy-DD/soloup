//! soloup-solver —— 结算编排 / 派生只读面 / 统一写入口。
//! 对应 TS `@soloup/solver`。service 层（createSolver）在 service.rs。

pub mod derive;
pub mod errors;
pub mod ledger;
pub mod params;
pub mod service;

pub use errors::{SolverError, SolverErrorCode};
pub use service::Solver;
