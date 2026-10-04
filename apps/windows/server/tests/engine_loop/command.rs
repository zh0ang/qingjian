//! 命令模式：自动识别命令候选、分类停用与模式关闭。
//!
//! 数据用随包 `assets/commands/`（15 分类），断言走文本与 `CandidateKind::Command` 标记。

use crate::support::*;
use qingjian_core::CandidateKind;
use qingjian_platform::{CommandConfig, CommandModeSetting};

/// 带随包命令库的 Router。
fn router_with_commands() -> Router {
    router_with_commands_config(CommandConfig::default())
}

fn router_with_commands_config(command: CommandConfig) -> Router {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let engine = assembly::assemble(&AssemblySpec {
        bundled_commands_dir: Some(root.join("assets/commands")),
        command,
        ..AssemblySpec::new(root.join("assets/sample/dict.tsv"))
    })
    .expect("assemble engine with command library");
    let mut router = Router::new(engine, RouterConfig::default());
    open_session(&mut router, SESSION, None);
    router
}

/// 帧里命令类型的候选文本。
fn command_texts(frame: &Frame) -> Vec<&str> {
    frame
        .candidates
        .items
        .iter()
        .filter(|c| c.kind == CandidateKind::Command)
        .map(|c| c.text.as_str())
        .collect()
}

/// 中文模式敲 `fas`：命令候选插到最前，完整命令可见。
#[test]
fn fastboot_prefix_yields_command_candidates_first() {
    let mut router = router_with_commands();
    let (_, _, frame) = type_letters(&mut router, "fas");
    let commands = command_texts(&frame);
    assert!(
        !commands.is_empty(),
        "fas 应命中 fastboot 分类：{commands:?} / 全部候选 {:?}",
        candidate_texts(&frame)
    );
    assert_eq!(commands[0], "fastboot devices");
    assert!(
        commands.iter().any(|c| *c == "fastboot reboot"),
        "应有 fastboot reboot：{commands:?}"
    );
    assert_eq!(
        candidate_texts(&frame)[0],
        "fastboot devices",
        "命令候选应排在最前"
    );
}

/// 不命中命令库的纯英文前缀不掺命令候选，拼音照常。
#[test]
fn unmatched_english_prefix_keeps_pinyin_candidates() {
    let mut router = router_with_commands();
    let (_, _, frame) = type_letters(&mut router, "nihao");
    assert!(
        command_texts(&frame).is_empty(),
        "nihao 不应命中任何命令前缀：{:?}",
        command_texts(&frame)
    );
    assert!(!candidate_texts(&frame).is_empty(), "拼音候选应还在");
}

/// 总开关关闭：一律不出命令候选。
#[test]
fn disabled_switch_suppresses_command_candidates() {
    let mut router = router_with_commands_config(CommandConfig {
        enabled: false,
        ..CommandConfig::default()
    });
    let (_, _, frame) = type_letters(&mut router, "fas");
    assert!(command_texts(&frame).is_empty());
}

/// `mode = off` 与总开关等价：不出命令候选。
#[test]
fn mode_off_suppresses_command_candidates() {
    let mut router = router_with_commands_config(CommandConfig {
        mode: CommandModeSetting::Off,
        ..CommandConfig::default()
    });
    let (_, _, frame) = type_letters(&mut router, "fas");
    assert!(command_texts(&frame).is_empty());
}

/// 停用分类：fastboot 被过滤，adb 仍可命中。
#[test]
fn disabled_category_filters_its_commands_only() {
    let mut router = router_with_commands_config(CommandConfig {
        disabled_categories: vec!["fastboot".to_owned()],
        ..CommandConfig::default()
    });
    let (_, _, frame) = type_letters(&mut router, "fas");
    assert!(command_texts(&frame).is_empty(), "fastboot 分类应被过滤");
    press(&mut router, function_key(0x1B));
    let (_, _, frame) = type_letters(&mut router, "adb");
    let commands = command_texts(&frame);
    assert!(
        commands.iter().any(|c| *c == "adb shell"),
        "adb 分类仍应命中：{commands:?}"
    );
}

/// 选择命令候选：数字键上屏完整命令文本。
#[test]
fn selecting_command_commits_full_text() {
    let mut router = router_with_commands();
    let (_, _, frame) = type_letters(&mut router, "fas");
    let slot = slot_of(&frame, "fastboot reboot");
    let (_, commit, _) = press(&mut router, digit(slot));
    assert_eq!(commit.as_deref(), Some("fastboot reboot"));
}

/// 英文模式下同样自动识别命令前缀。
#[test]
fn english_mode_also_offers_command_candidates() {
    let mut router = router_with_commands();
    let (_, _, frame) = type_english(&mut router, "fas");
    assert!(
        command_texts(&frame).iter().any(|c| *c == "fastboot reboot"),
        "英文模式也应有命令候选：{:?}",
        command_texts(&frame)
    );
}
