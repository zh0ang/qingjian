//! 命令模式：模块入口、状态机与对外 API。
//!
//! 输入命令前缀（如 `fas`）时，候选区展示完整命令补全（如 `fastboot reboot`）。
//! 数据来自 [`CommandDb`]（`assets/commands/*.tsv`），候选以
//! [`CandidateKind::Command`](crate::candidate::CandidateKind::Command) 类型进入候选列表。

pub mod db;
pub mod engine;
pub mod entry;

pub use db::CommandDb;
pub use engine::MAX_COMMAND_CANDIDATES;
pub use entry::CommandEntry;

/// 命令模式状态机。
///
/// - [`Auto`](CommandMode::Auto)：自动识别——输入纯英文 / 命令字符且前缀命中命令库时出命令候选；
/// - [`Manual`](CommandMode::Manual)：快捷键强制开启（与 Auto 同样的匹配逻辑，保留给后续
///   模糊匹配、场景感知等强意图扩展）；
/// - [`Off`](CommandMode::Off)：关闭，一律不出命令候选。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandMode {
    #[default]
    Auto,
    Manual,
    Off,
}

impl CommandMode {
    /// 当前是否允许出命令候选。
    pub fn is_active(self) -> bool {
        !matches!(self, Self::Off)
    }

    /// 快捷键轮换：Auto → Manual → Off → Auto。
    pub fn toggle(self) -> Self {
        match self {
            Self::Auto => Self::Manual,
            Self::Manual => Self::Off,
            Self::Off => Self::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandMode;

    #[test]
    fn default_is_auto() {
        assert_eq!(CommandMode::default(), CommandMode::Auto);
        assert!(CommandMode::Auto.is_active());
        assert!(CommandMode::Manual.is_active());
        assert!(!CommandMode::Off.is_active());
    }

    #[test]
    fn toggle_cycles() {
        let mut mode = CommandMode::Auto;
        mode = mode.toggle();
        assert_eq!(mode, CommandMode::Manual);
        mode = mode.toggle();
        assert_eq!(mode, CommandMode::Off);
        mode = mode.toggle();
        assert_eq!(mode, CommandMode::Auto);
    }
}
