//! 命令库：TSV 加载与前缀索引（Trie）。
//!
//! 数据文件位于 `assets/commands/`，每分类一个 `.tsv`，每行
//! `命令文本 \t 分类 \t 说明`。加载时按分类过滤未启用项，索引在启动时
//! 构建，匹配复杂度 O(前缀长度)。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::entry::CommandEntry;

/// 前缀 Trie 节点：每个节点记录「以这个前缀开头的所有条目」的索引。
#[derive(Debug, Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    /// 该前缀（含路径上每个节点）下所有条目的索引，按插入顺序。
    indexes: Vec<usize>,
}

/// 命令库：条目列表 + 前缀索引 + 启用分类。
#[derive(Debug, Default)]
pub struct CommandDb {
    entries: Vec<CommandEntry>,
    root: TrieNode,
    /// 显式启用的分类；`None` 表示全部启用。
    enabled: Option<Vec<String>>,
    /// 加载过的分类（按文件出现顺序）。
    categories: Vec<String>,
}

impl CommandDb {
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载目录下所有 `*.tsv` 文件，分类取文件名（不含扩展名）。
    ///
    /// 跳过未启用分类与损坏 / 无有效条目的文件；返回 `(加载条目数, 失败文件数)`。
    /// 失败只记日志、不影响输入法主功能。
    pub fn load_dir(&mut self, dir: &Path) -> Result<(usize, usize), String> {
        let mut loaded = 0;
        let mut failed = 0;
        let mut entries = Vec::new();
        let mut categories = Vec::new();
        let dir = fs::read_dir(dir).map_err(|e| format!("读取命令库目录失败: {e}"))?;
        let mut paths: Vec<_> = dir
            .filter_map(|entry| entry.ok().map(|d| d.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tsv"))
            .collect();
        paths.sort();
        for path in paths {
            let category = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            if let Some(enabled) = &self.enabled
                && !enabled.contains(&category)
            {
                continue;
            }
            match Self::load_file_into(&mut entries, &path, &category) {
                Ok(count) if count > 0 => {
                    loaded += count;
                    categories.push(category);
                }
                Ok(_) => {
                    failed += 1;
                    tracing::warn!(category, "命令库文件没有有效条目");
                }
                Err(message) => {
                    failed += 1;
                    tracing::warn!(%message, "命令库文件跳过");
                }
            }
        }
        self.entries = entries;
        self.categories = categories;
        self.rebuild();
        Ok((loaded, failed))
    }

    /// 追加加载单个 TSV 文件（设置页导入自定义命令用）。
    pub fn load_file(&mut self, path: &Path, category: &str) -> Result<usize, String> {
        let mut entries = Vec::new();
        let count = Self::load_file_into(&mut entries, path, category)?;
        if !self.categories.contains(&category.to_string()) {
            self.categories.push(category.to_string());
        }
        self.entries.extend(entries);
        self.rebuild();
        Ok(count)
    }

    /// 运行时重设启用分类（`None` = 全部启用）并重建索引；热加载用。
    ///
    /// 只调整索引过滤，不删除已装载条目；重新启用某分类时原条目仍可用。
    pub fn set_enabled_categories(&mut self, enabled: Option<Vec<String>>) {
        self.enabled = enabled;
        self.rebuild();
    }

    pub fn enabled_categories(&self) -> Option<&[String]> {
        self.enabled.as_deref()
    }

    /// 已加载分类（按文件出现顺序）。
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    pub fn entries(&self) -> &[CommandEntry] {
        &self.entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn count_by_category(&self, category: &str) -> usize {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .count()
    }

    /// 前缀查询：返回命令文本（小写比较）以 `prefix` 开头的条目，最多 `limit` 条。
    pub fn search(&self, prefix: &str, limit: usize) -> Vec<&CommandEntry> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }
        node.indexes
            .iter()
            .take(limit)
            .filter_map(|&index| self.entries.get(index))
            .collect()
    }

    fn load_file_into(
        out: &mut Vec<CommandEntry>,
        path: &Path,
        category: &str,
    ) -> Result<usize, String> {
        let text =
            fs::read_to_string(path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let mut count = 0;
        for line in text.lines() {
            if let Some(entry) = CommandEntry::parse_tsv(line) {
                // 每分类一个文件：行内分类列与文件名不一致视为脏数据，跳过。
                if entry.category != category {
                    tracing::warn!(
                        category,
                        line_category = %entry.category,
                        "命令库文件内分类与文件名不一致，跳过该行"
                    );
                    continue;
                }
                out.push(entry);
                count += 1;
            }
        }
        Ok(count)
    }

    /// 测试辅助：直接塞条目并重建索引。
    #[cfg(test)]
    pub(crate) fn set_for_test(&mut self, entries: Vec<CommandEntry>) {
        self.entries = entries;
        self.rebuild();
    }

    fn rebuild(&mut self) {
        self.root = TrieNode::default();
        // 先按索引逐条克隆命令串再插入，避免 iter() 不可变借用与 insert 可变借用冲突。
        // 未启用分类的条目不进入索引，但保留在 entries 里供重新启用。
        for index in 0..self.entries.len() {
            let entry = &self.entries[index];
            if let Some(list) = &self.enabled
                && !list.contains(&entry.category)
            {
                continue;
            }
            let command = entry.command.clone();
            self.insert(&command, index);
        }
    }

    fn insert(&mut self, command: &str, index: usize) {
        let mut node = &mut self.root;
        for ch in command.to_ascii_lowercase().chars() {
            node = node.children.entry(ch).or_default();
            node.indexes.push(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandDb;

    /// 写一个临时 TSV 并加载，验证前缀匹配与分类计数。
    #[test]
    fn load_and_search() {
        let dir = std::env::temp_dir().join("qingjian-command-db-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fastboot.tsv");
        std::fs::write(
            &file,
            "fastboot reboot\tfastboot\t重启设备到系统\n\
             fastboot flash recovery <img>\tfastboot\t刷入 recovery 镜像\n\
             # 注释行\n\
             adb shell\tadb\t进入设备 shell\n",
        )
        .unwrap();
        let mut db = CommandDb::new();
        let (loaded, failed) = db.load_dir(&dir).unwrap();
        assert_eq!(loaded, 2, "只认 fastboot 分类文件里的 2 条有效条目");
        assert_eq!(failed, 0);
        std::fs::remove_dir_all(&dir).unwrap();

        let hits = db.search("fastboot", 10);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].command, "fastboot reboot");

        let hits = db.search("fas", 10);
        assert_eq!(hits.len(), 2, "前缀 fas 命中全部 fastboot 条目");

        let hits = db.search("adb", 10);
        assert!(hits.is_empty(), "adb.tsv 不存在，不该命中");

        assert_eq!(db.count_by_category("fastboot"), 2);
    }

    #[test]
    fn prefix_progression() {
        use crate::command::entry::CommandEntry;
        let mut db = CommandDb::new();
        db.set_for_test(vec![
            CommandEntry {
                command: "git status".into(),
                category: "git".into(),
                description: "查看状态".into(),
            },
            CommandEntry {
                command: "git commit -m <msg>".into(),
                category: "git".into(),
                description: "提交".into(),
            },
        ]);
        assert_eq!(db.search("g", 10).len(), 2);
        assert_eq!(db.search("gi", 10).len(), 2);
        assert_eq!(db.search("git c", 10).len(), 1);
        assert_eq!(db.search("git s", 10).len(), 1);
        assert_eq!(db.search("x", 10).len(), 0);
    }

    #[test]
    fn disable_category_rebuilds() {
        use crate::command::entry::CommandEntry;
        let mut db = CommandDb::new();
        db.set_for_test(vec![
            CommandEntry {
                command: "adb devices".into(),
                category: "adb".into(),
                description: "列出设备".into(),
            },
            CommandEntry {
                command: "ls -al".into(),
                category: "linux".into(),
                description: "列出目录".into(),
            },
        ]);
        db.categories = vec!["adb".into(), "linux".into()];
        assert_eq!(db.search("adb", 10).len(), 1);
        assert_eq!(db.search("ls", 10).len(), 1);

        db.set_enabled_categories(Some(vec!["linux".into()]));
        assert_eq!(db.search("adb", 10).len(), 0, "停用 adb 后不再命中");
        assert_eq!(db.search("ls", 10).len(), 1);

        db.set_enabled_categories(None);
        assert_eq!(db.search("adb", 10).len(), 1, "重新全部启用");
    }
}
