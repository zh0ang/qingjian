//! 命令模式：总开关、识别方式与禁用的命令分类。

use serde::{Deserialize, Serialize};

use qingjian_core::command::CommandMode;

/// 命令模式识别方式（配置里的字符串值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandModeSetting {
    /// 自动识别：纯英文 / 命令字符且前缀命中命令库时出命令候选。
    Auto,
    /// 快捷键强制开启（与 Auto 同样的匹配逻辑，保留给模糊匹配、场景感知等强意图扩展）。
    Manual,
    /// 关闭，一律不出命令候选。
    Off,
}

impl Default for CommandModeSetting {
    fn default() -> Self {
        Self::Auto
    }
}

impl CommandModeSetting {
    /// 设置页下拉的选项顺序。
    pub const ALL: [Self; 3] = [Self::Auto, Self::Manual, Self::Off];

    /// 下拉里显示的标签。
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "自动识别",
            Self::Manual => "手动开关",
            Self::Off => "关闭",
        }
    }

    /// 转成引擎用的状态机。
    pub fn mode(self) -> CommandMode {
        match self {
            Self::Auto => CommandMode::Auto,
            Self::Manual => CommandMode::Manual,
            Self::Off => CommandMode::Off,
        }
    }

    /// 配置里的字符串值（`[command] mode` 落盘用，与 serde snake_case 一致）。
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
            Self::Off => "off",
        }
    }
}

/// `[command]` 配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandConfig {
    /// 命令模式总开关：false 时整条命令线关，候选不挂命令补全。
    pub enabled: bool,

    /// 识别方式：auto 自动识别 / manual 手动开关 / off 关闭。
    pub mode: CommandModeSetting,

    /// 关掉的命令分类（文件名，不含扩展名）；随包的 `assets/commands/*.tsv` 与用户
    /// `commands/` 下的 .tsv 都会加载，这里列出要关掉的分类。
    pub disabled_categories: Vec<String>,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: CommandModeSetting::Auto,
            disabled_categories: Vec::new(),
        }
    }
}

/// 命令模式的分类开关：`None` 表示全部启用。
impl CommandConfig {
    /// 启用分类列表；`disabled_categories` 为空时返回 `None`（全部启用）。
    pub fn enabled_categories(&self, categories: &[String]) -> Option<Vec<String>> {
        if self.disabled_categories.is_empty() {
            return None;
        }
        let disabled: std::collections::HashSet<&String> =
            self.disabled_categories.iter().collect();
        Some(
            categories
                .iter()
                .filter(|category| !disabled.contains(*category))
                .cloned()
                .collect(),
        )
    }

    /// 某个分类（文件名，不含扩展名）是否启用。
    pub fn is_enabled(&self, category: &str) -> bool {
        !self.disabled_categories.iter().any(|c| c == category)
    }

    /// 设置页下拉里当前 `mode` 的下标（与 [`CommandModeSetting::ALL`] 顺序一致）。
    pub fn mode_index(&self) -> usize {
        CommandModeSetting::ALL
            .iter()
            .position(|mode| *mode == self.mode)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto_and_all_enabled() {
        let config = CommandConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, CommandModeSetting::Auto);
        assert!(config.disabled_categories.is_empty());
        assert_eq!(config.enabled_categories(&["adb".into(), "git".into()]), None);
    }

    #[test]
    fn disabled_categories_filter() {
        let config = CommandConfig {
            enabled: true,
            mode: CommandModeSetting::Manual,
            disabled_categories: vec!["adb".into(), "edl".into()],
        };
        let enabled = config
            .enabled_categories(&["adb".into(), "fastboot".into(), "edl".into(), "git".into()])
            .unwrap();
        assert_eq!(enabled, vec!["fastboot".to_string(), "git".to_string()]);
    }

    #[test]
    fn mode_roundtrip_via_toml() {
        use crate::config::Config;
        // `[command]` 是完整配置的子表，从完整 Config 解析更贴近真实场景。
        let full: Config = toml::from_str("[command]\nmode = \"manual\"\n").unwrap();
        assert_eq!(full.command.mode, CommandModeSetting::Manual);
        // 完整配置对象往返：序列化再反序列化后 mode 不变。
        let text = toml::to_string(&full).unwrap();
        let again: Config = toml::from_str(&text).unwrap();
        assert_eq!(again.command.mode, CommandModeSetting::Manual);
    }
}
