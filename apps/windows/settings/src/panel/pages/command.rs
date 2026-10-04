//! 「命令」页：命令模式总开关、触发模式、15 个命令分类的开关与导入命令库。
//! 开关写 `[command] enabled / mode`，分类开关写 `[command] disabled_categories`（列关掉的，与词库同形）。
//! 导入的 .tsv 放进 `%APPDATA%\Qingjian\commands`，随包命令库在随包根的 `assets/commands/`。

use std::path::PathBuf;

use qingjian_core::command::CommandDb;
use qingjian_platform::CommandModeSetting;
use windows_reactor::*;

use crate::panel::controls::{check_row, feedback, field, note, page, repo_resource};
use crate::panel::{Message, Settings};

/// 随包命令库目录：随包根下的 `assets/commands/`，与 Server 装配同款。
const BUNDLED_DIR: &str = "assets/commands";

/// 用户导入的命令库目录 `%APPDATA%\Qingjian\commands`。
fn user_dir(settings: &Settings) -> PathBuf {
    settings.data_dir().join("commands")
}

/// 分类的中文名（文件 stem → 界面标签），没列到的直接用 stem。
fn category_label(stem: &str) -> String {
    let label = match stem {
        "fastboot" => "fastboot",
        "adb" => "adb",
        "edl" => "edl（高通 9008）",
        "ssh" => "ssh / scp / sftp / rsync",
        "network" => "网络",
        "disk" => "磁盘",
        "dev" => "开发",
        "ops" => "运维",
        "compress" => "压缩",
        "terminal" => "终端",
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        "git" => "Git",
        "custom" => "自定义",
        _ => stem,
    };
    format!("{label}（{stem}）")
}

/// 列一个目录里的命令库文件，按 stem 排序；返回 (stem, path, 条数)。
fn list_dir(dir: &PathBuf) -> Vec<(String, PathBuf, usize)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(String, PathBuf)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("tsv") {
                return None;
            }
            let stem = path.file_stem().and_then(|s| s.to_str())?.to_owned();
            Some((stem, path))
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
        .into_iter()
        .map(|(stem, path)| {
            let count = CommandDb::new().load_file(&path, &stem).unwrap_or(0);
            (stem, path, count)
        })
        .collect()
}

/// 随包命令库：标「随包」，只给开关，不给移除。
fn bundled_list(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let Some(dir) = repo_resource(BUNDLED_DIR) else {
        return note("没找到随包命令库目录（随包根的 assets\\commands\\），随包命令库装好后这里会列出来。");
    };
    let files = list_dir(&dir);
    if files.is_empty() {
        return note("随包命令库目录是空的。");
    }
    let mut rows: Vec<KeyedView> = Vec::with_capacity(files.len());
    for (stem, _path, count) in files {
        let enabled = settings.config.command.is_enabled(&stem);
        let label = format!(
            "{} · {count} 条 · 随包",
            category_label(&stem)
        );
        rows.push(check_row(
            &stem.clone(),
            label,
            enabled,
            false,
            move |on| Message::ToggleCommandCategory(stem.clone(), on),
            None,
            context,
        ));
    }
    StackPanel::new().spacing(6.0).keyed_children(rows)
}

/// 自己导入的命令库：可开关、可移除（挪进 commands\removed，不真删）。
fn user_list(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let dir = user_dir(settings);
    let files = list_dir(&dir);
    if files.is_empty() {
        return note("还没有导入命令库。点下面「导入命令库」加一个 .tsv，或把文件放进 %APPDATA%\\Qingjian\\commands。");
    }
    let mut rows: Vec<KeyedView> = Vec::with_capacity(files.len());
    for (stem, _path, count) in files {
        let enabled = settings.config.command.is_enabled(&stem);
        let label = format!("{} · {count} 条", category_label(&stem));
        let remove = Message::RemoveCommandFile(stem.clone());
        rows.push(check_row(
            &stem.clone(),
            label,
            enabled,
            false,
            move |on| Message::ToggleCommandCategory(stem.clone(), on),
            Some(remove),
            context,
        ));
    }
    StackPanel::new().spacing(6.0).keyed_children(rows)
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let body = StackPanel::new().spacing(12.0).children([
        note(
            "开着命令模式后，敲 fastboot、adb、ssh 这类命令前缀会在候选窗里直接补全子命令，回车上屏。改完自动生效。",
        ),
        field(
            "启用命令模式",
            "开着时输入自动识别为命令就出命令补全；关着时命令库完全不参与候选。",
            ToggleSwitch::new()
                .is_on(settings.config.command.enabled)
                .on_toggled(context.callback(Message::CommandEnabled)),
        ),
        field(
            "触发方式",
            "自动：像 fastboot 这样命中命令库前缀就补全；手动：按快捷键（建议 Ctrl+` 或 F9）强制开关后只补命令；关闭：完全不参与。",
            ComboBox::new()
                .items_source(CommandModeSetting::ALL.map(|mode| mode.label()))
                .selected_index(settings.config.command.mode_index())
                .on_selection_changed(context.callback(Message::CommandMode)),
        ),
        TextBlock::new()
            .text("命令分类")
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
        note("按分类开关命令库；关掉的分类即使命中也不出补全。随包命令库共 15 个分类，自带的如下。"),
        bundled_list(settings, context),
        user_list(settings, context),
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children((
                Button::new()
                    .on_click(context.message(Message::ImportCommand))
                    .content("导入命令库…"),
                note(
                    "接受三列的 .tsv（命令、分类、说明，说明可缺省）。分类名取文件名，导入后同名覆盖；文件由你自己取得。",
                ),
            )),
        feedback(&settings.notice),
    ]);
    page("命令", body)
}

/// 文件选择器选一个命令库 .tsv，校验格式后放进用户命令库目录；成功统计进 note，失败进红字。
pub(crate) fn import(settings: &mut Settings) {
    let Some(source) = rfd::FileDialog::new()
        .add_filter("命令库", &["tsv"])
        .add_filter("所有文件", &["*"])
        .set_title("导入命令库")
        .pick_file()
    else {
        return;
    };
    let Some(stem) = source.file_stem().and_then(|s| s.to_str()) else {
        settings.notice.fail("命令库文件名不对。".to_owned());
        return;
    };
    // 先校验格式：三列 \t 分隔，空行 / # 注释跳过；坏了就不落盘。
    let count = match CommandDb::new().load_file(&source, stem) {
        Ok(count) => count,
        Err(error) => {
            settings.notice.fail(format!("命令库格式不对（{error}）：要三列 \t 分隔的 .tsv。"));
            return;
        }
    };
    let dest = user_dir(settings).join(format!("{stem}.tsv"));
    if let Err(error) = std::fs::create_dir_all(dest.parent().expect("用户命令库目录必有父目录")) {
        settings.notice.fail(format!("建用户命令库目录失败：{error}"));
        return;
    }
    if let Err(error) = std::fs::copy(&source, &dest) {
        settings.notice.fail(format!("复制命令库失败：{error}"));
        return;
    }
    settings.notice.succeed(format!("已导入命令库「{stem}」：{count} 条命令。"));
}

/// 挪进 `commands\removed`，不真删（与词库一致）。
pub(crate) fn remove_file(settings: &Settings, stem: &str) {
    let dir = user_dir(settings);
    let Some((_, path, _)) = list_dir(&dir).into_iter().find(|(name, _, _)| name == stem) else {
        return;
    };
    let removed = dir.join("removed");
    if let Err(error) = std::fs::create_dir_all(&removed) {
        eprintln!("建 removed 目录失败: {error}");
        return;
    }
    if let Some(file_name) = path.file_name()
        && let Err(error) = std::fs::rename(&path, removed.join(file_name))
    {
        eprintln!("移除命令库 {stem} 失败: {error}");
    }
}
