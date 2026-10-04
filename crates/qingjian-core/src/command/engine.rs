//! 命令匹配引擎：把输入串变成 [`CandidateKind::Command`] 候选。
//!
//! 触发判定走「纯 ASCII 命令字符 + 前缀命中命令库」双条件：中文拼音输入
//! 里混入的字母串若前缀没命中命令库，一律回落正常拼音，不影响原有候选。

use super::db::CommandDb;
use crate::candidate::{Candidate, CandidateKind};

/// 命令补全一次最多给几条候选。
pub const MAX_COMMAND_CANDIDATES: usize = 9;

/// 命令模式的触发判定：输入是否像命令前缀。
///
/// 允许 `a-zA-Z0-9` 与命令常见的 `-` `_` `.` `/` `:` 空格（支持 `fastboot flash`、
/// `open .` 这类含分隔符与子命令的多段前缀）；含中文 / 标点 / 表达式键一律不触发。
pub fn is_command_like(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | ' ')
        })
}

/// 生成命令候选：查前缀 Trie，命中返回 Command 类型候选（说明放 `reading`）。
pub fn command_candidates(db: &CommandDb, input: &str, limit: usize) -> Vec<Candidate> {
    let key = input.trim();
    if key.is_empty() || !is_command_like(key) {
        return Vec::new();
    }
    db.search(&key.to_ascii_lowercase(), limit)
        .into_iter()
        .map(|entry| Candidate {
            text: entry.command.clone(),
            kind: CandidateKind::Command,
            syllables: Vec::new(),
            reading: Some(entry.description.clone()),
            translation: None,
            aux_code: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{command_candidates, is_command_like, MAX_COMMAND_CANDIDATES};
    use crate::command::entry::CommandEntry;
    use crate::candidate::CandidateKind;

    fn sample_db() -> crate::command::db::CommandDb {
        let mut db = crate::command::db::CommandDb::new();
        db.load_file(std::path::Path::new("assets/commands/_none.tsv"), "none").ok();
        // 直接手工构造，避免单测依赖仓库数据文件
        db.set_for_test(vec![
            CommandEntry {
                command: "fastboot reboot".into(),
                category: "fastboot".into(),
                description: "重启到系统".into(),
            },
            CommandEntry {
                command: "fastboot flash recovery <img>".into(),
                category: "fastboot".into(),
                description: "刷入 recovery 镜像".into(),
            },
            CommandEntry {
                command: "adb shell".into(),
                category: "adb".into(),
                description: "进入设备 shell".into(),
            },
            CommandEntry {
                command: "open .".into(),
                category: "custom".into(),
                description: "在当前目录打开资源管理器".into(),
            },
        ]);
        db
    }

    #[test]
    fn trigger_detection() {
        assert!(is_command_like("fas"));
        assert!(is_command_like("fastboot r"));
        assert!(is_command_like("adb-sync"));
        assert!(is_command_like("open ."));
        assert!(!is_command_like(""));
        assert!(!is_command_like("你好"));
        assert!(!is_command_like("fastboot?"));
        assert!(!is_command_like("a+b"));
    }

    #[test]
    fn candidates_hit() {
        let db = sample_db();
        let hits = command_candidates(&db, "fas", MAX_COMMAND_CANDIDATES);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].kind, CandidateKind::Command);
        assert_eq!(hits[0].text, "fastboot reboot");
        assert_eq!(
            hits[0].reading.as_deref(),
            Some("重启到系统"),
            "说明放 reading，候选窗口可展示"
        );
    }

    #[test]
    fn candidates_subcommand_prefix() {
        let db = sample_db();
        let hits = command_candidates(&db, "fastboot flash", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "fastboot flash recovery <img>");
    }

    #[test]
    fn candidates_miss_falls_back() {
        let db = sample_db();
        let hits = command_candidates(&db, "xyz", 10);
        assert!(hits.is_empty(), "未命中命令库时不出命令候选");
        // 大写输入归一化
        let hits = command_candidates(&db, "ADB", 10);
        assert_eq!(hits.len(), 1);
    }
}
