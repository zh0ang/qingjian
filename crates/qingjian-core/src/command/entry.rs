//! 命令条目：命令文本、分类、说明。

/// 一条命令补全数据。
///
/// 从 `assets/commands/*.tsv` 加载，每行三列：`命令文本 \t 分类 \t 说明`。
/// 命令文本是上屏内容（如 `fastboot reboot`、`adb shell`、`ls -al`），
/// 分类与文件对应（fastboot / adb / edl / ssh / network / disk / dev / ops /
/// compress / terminal / linux / windows / macos / git / custom）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntry {
    /// 完整命令文本，选中后原样上屏。
    pub command: String,
    /// 所属分类（对应 TSV 文件名）。
    pub category: String,
    /// 简要说明，设置页与候选渲染可展示。
    pub description: String,
}

impl CommandEntry {
    /// 从一行 TSV 解析命令条目。
    ///
    /// 格式：`命令文本 \t 分类 \t 说明`。
    /// - 空行与以 `#` 开头的注释行返回 `None`；
    /// - 命令文本或分类为空返回 `None`；
    /// - 说明允许为空（第三列可缺省）。
    pub fn parse_tsv(line: &str) -> Option<Self> {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            return None;
        }
        let mut parts = line.splitn(3, '\t');
        let command = parts.next()?.trim().to_string();
        let category = parts.next()?.trim().to_string();
        let description = parts.next().unwrap_or("").trim().to_string();
        if command.is_empty() || category.is_empty() {
            return None;
        }
        Some(Self {
            command,
            category,
            description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CommandEntry;

    #[test]
    fn parse_normal_line() {
        let entry = CommandEntry::parse_tsv("fastboot reboot\tfastboot\t重启设备到系统").unwrap();
        assert_eq!(entry.command, "fastboot reboot");
        assert_eq!(entry.category, "fastboot");
        assert_eq!(entry.description, "重启设备到系统");
    }

    #[test]
    fn parse_missing_description() {
        let entry = CommandEntry::parse_tsv("adb shell\tadb").unwrap();
        assert_eq!(entry.command, "adb shell");
        assert_eq!(entry.category, "adb");
        assert_eq!(entry.description, "");
    }

    #[test]
    fn parse_trims_whitespace() {
        let entry = CommandEntry::parse_tsv("  ls -al \t linux \t 列出目录内容  ").unwrap();
        assert_eq!(entry.command, "ls -al");
        assert_eq!(entry.category, "linux");
        assert_eq!(entry.description, "列出目录内容");
    }

    #[test]
    fn parse_skips_invalid_lines() {
        assert!(CommandEntry::parse_tsv("").is_none());
        assert!(CommandEntry::parse_tsv("   ").is_none());
        assert!(CommandEntry::parse_tsv("# 注释").is_none());
        assert!(CommandEntry::parse_tsv("\tlinux\t说明").is_none());
        assert!(CommandEntry::parse_tsv("ls -al\t\t说明").is_none());
    }
}
