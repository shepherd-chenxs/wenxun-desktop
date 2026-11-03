// 命令层按领域拆分为子模块；mod.rs 只负责宏定义与再导出，
// main.rs 中的 wenxun::commands::xxx 路径保持不变。
//
// 注意：lock_db! 是 macro_rules 文本作用域，必须先于 mod 声明定义才能在子模块内使用。
macro_rules! lock_db {
    ($state:expr) => {
        match $state.db.lock() {
            Ok(db) => db,
            Err(e) => return Err(format!("数据库锁定失败: {}", e)),
        }
    };
}

pub mod accounts;
pub mod ai;
pub mod articles;
pub mod basic;
pub mod bloggers;
pub mod tasks;
pub mod topics;
pub mod wechat;
pub mod workshop;

pub use accounts::*;
pub use ai::*;
pub use articles::*;
pub use basic::*;
pub use bloggers::*;
pub use tasks::*;
pub use topics::*;
pub use wechat::*;
pub use workshop::*;
