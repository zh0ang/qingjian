---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: 5445b6d8cfaf9f844a871d5a380ffd99_d00a77fbbfea11f1887c525400de85a5
    ReservedCode1: 7OLB/ivOk6aAaATLrlgvL0ovyrFYUogQccg5i6uz1uEsMjVZaOxzUCVGUn5aZ9tgrhZsxIKtgbxgXvlpk9s1LgZ1/4ahqw4PYvqIAvP9FqnK/CqV0rtnOVmGi4j6rpqT7qObc+yOZy5kRDXjrM/QHoKq969d11sv6IhWsfoJIxWlIaCGphGq0evqRRE=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: 5445b6d8cfaf9f844a871d5a380ffd99_d00a77fbbfea11f1887c525400de85a5
    ReservedCode2: 7OLB/ivOk6aAaATLrlgvL0ovyrFYUogQccg5i6uz1uEsMjVZaOxzUCVGUn5aZ9tgrhZsxIKtgbxgXvlpk9s1LgZ1/4ahqw4PYvqIAvP9FqnK/CqV0rtnOVmGi4j6rpqT7qObc+yOZy5kRDXjrM/QHoKq969d11sv6IhWsfoJIxWlIaCGphGq0evqRRE=
---

<p align="center">
  <img src="assets/icon/qingjian-mark.svg" alt="青简竹简图标" height="108">
</p>

<h1 align="center">青简 Qingjian · 命令模式版</h1>

<p align="center"><strong>好好输入，顺便多认识一个词；输入命令，顺手把命令补全。</strong></p>

<p align="center">
  <a href="https://github.com/qingjian-team/qingjian"><img src="https://img.shields.io/badge/fork%20of-qingjian--team%2Fqingjian-blue" alt="fork of qingjian-team/qingjian"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" alt="License: GPL-3.0-or-later"></a>
  <a href="https://img.shields.io/badge/macOS-13%2B-blue"><img src="https://img.shields.io/badge/macOS-13%2B-blue" alt="macOS 13+"></a>
  <a href="https://img.shields.io/badge/Windows-10%2F11-blue"><img src="https://img.shields.io/badge/Windows-10%2F11-blue" alt="Windows 10/11"></a>
  <a href="https://img.shields.io/badge/Linux-Fcitx5-lightgrey"><img src="https://img.shields.io/badge/Linux-Fcitx5-lightgrey" alt="Linux Fcitx5"></a>
</p>

## 这个项目是什么

青简是一款开源输入法：像平常一样输入拼音、选择候选、写完整句，候选旁的译词让语言学习自然发生在日常输入里。

**本仓库是青简的派生修改版（fork）**，在保留上游完整输入能力的基础上，新增了面向开发与刷机场景的「命令模式」：输入 `fas` 就能直接补全出 `fastboot reboot` 这样的完整命令行。核心修改集中在 `crates/qingjian-core` 的命令模块与随包命令库，其余输入能力与上游保持一致，上游迭代可持续合入。

## 功能总览

| 能力 | 说明 |
| --- | --- |
| 拼音输入 | 全拼 / 简拼自动补分隔符，整句转换（bigram + Viterbi） |
| 双拼 | 小鹤、自然码、微软、搜狗四套方案，与全拼共享切分与整句 |
| 注音 / 英文 | 注音输入、英文模式候选 |
| 候选译词 | 候选旁直接显示词性与中英 / 中日译词，汉字注平假名，语言学习自然发生 |
| 纠错与模糊音 | 常见拼音纠错，z/zh、n/l、an/ang 等九组模糊音，整句转换同样生效 |
| Emoji | 内置 Emoji 表，输入即出 |
| 本地学习 | 词频与个人 n-gram 全部本地统计，崩溃不丢数据 |
| 命令模式（本仓库新增） | 15 类 387 条命令补全，详见下文 |

## 核心优势：命令模式，输入法里直接补齐命令行

平时敲命令要么翻笔记、要么先打一半再 Tab，本版青简把命令补全直接搬进了输入法候选区：

```text
fas
1  fastboot devices             列出连接的 fastboot 设备
2  fastboot reboot              重启设备到系统
3  fastboot flash boot <img>    刷入 boot 分区镜像
4  fastboot oem unlock          解锁 bootloader
```

- **自动识别**：默认 Auto 模式，输入纯命令字符且前缀命中命令库即自动出命令候选，命令候选永远排在候选区最前；
- **不影响正常输入**：中文拼音里混入的字母串若前缀未命中命令库，自动回落为普通拼音候选，输入毫无感知；
- **支持多段前缀**：可输入 `fastboot flash`、`adb shell` 这类带空格的多段前缀继续收窄候选；
- **候选带说明**：每条命令候选旁直接显示用途（如“刷入 boot 分区镜像”），不用猜这条命令是干嘛的；
- **快捷键切换**：Auto（自动）→ Manual（强制命令模式）→ Off（关闭）循环切换，按需控制是否出命令候选。

### 内置命令库：15 类 387 条，纯文本可扩展

| 分类 | 覆盖内容 |
| --- | --- |
| fastboot | devices / reboot / reboot-bootloader / flash boot / flash recovery / flash vbmeta / oem unlock 等 |
| adb | shell / push / pull / logcat / reverse / install / devices 等 |
| edl | 9008 深度刷机：解锁 bootloader、擦除/读写分区、打 GPT 表、执行刷机 XML 脚本等 |
| ssh | 连接、端口转发、免密登录等 |
| git | 常用 git 命令 |
| linux / macos / windows / terminal | 各系统常用终端命令 |
| network / disk / dev / compress / ops | 网络、磁盘、设备、压缩、运维场景命令 |
| custom | 用户自定义命令库 |

命令库是**纯文本、可扩展**的：位于 `assets/commands/*.tsv`，每行一条，格式为「命令\t分类\t说明」，加一行就是一条新命令，重新打包即可生效。`custom.tsv` 专为存放你自己的高频命令，例如：

```text
deploy	test	执行测试环境部署脚本
tf	test	实时跟踪日志（tail -f）
```

同时在设置页侧栏的「命令」入口里，你可以：按分类启用 / 停用（fastboot、adb、edl、ssh、git、custom…共 15 类）、导入自己的 `custom.tsv`、随时关闭整个命令模式。

## 使用场景示例

**刷机 / 解锁（fastboot + edl）**——手机、车机解锁刷机的常客：

```text
fas     →  fastboot devices / fastboot reboot / fastboot flash boot <img> / fastboot oem unlock …
edl     →  9008 深度刷机：解锁 bootloader、擦除 / 读写分区、打 GPT 表、执行刷机 XML 脚本 …
```

**远程管理（ssh）**：

```text
ssh     →  ssh user@host / ssh -L 8080:localhost:80 / ssh-keygen / scp file user@host:path …
```

**版本管理（git）**：

```text
git log --oneline     简洁提交历史
git reset --hard HEAD~1   回退一个提交
git stash              暂存工作区改动
```

## 与上游的关系

- 本仓库由 [qingjian-team/qingjian](https://github.com/qingjian-team/qingjian) 派生修改而来；
- 命令模式为本仓库**新增功能**，上游不包含；拼音输入、整句输入、候选译词、本地统计等其余能力与上游保持一致；
- 代码沿用上游的 [GPL-3.0-or-later](LICENSE) 许可，保留上游版权与许可声明，并按许可要求公开全部源码。

## 项目结构

| 路径 | 说明 |
| --- | --- |
| `crates/qingjian-core` | 核心引擎：拼音 / 双拼 / 注音 / 整句 / 模糊音 / 命令模式等 |
| `crates/qingjian-core/src/command/` | 命令模式模块（本仓库新增） |
| `assets/commands/*.tsv` | 随包命令库（15 分类，纯文本） |
| `crates/qingjian-format` | 二进制数据容器 `.qj`，词库 / 语言模型 mmap 零拷贝加载 |
| `crates/qingjian-translate` | 候选译词（词性 + 中英 / 中日） |
| `crates/qingjian-learning` | 本地学习与统计 |
| `apps/windows` | Windows TSF 输入法、后台服务、设置程序与安装包 |
| `apps/macos` | macOS IMK 输入法 |
| `apps/linux` | Linux Fcitx5 版本 |
| `apps/cli` | CLI 测试工具 |
| `tools/` | 词库转换 / 打包 / 评估等工具 |

## 从源码构建

需要 Rust 工具链（仓库固定在 1.96.0，见 `rust-toolchain.toml`）。

**Windows（MSVC 工具链）**：

```powershell
# 构建三个产物：TSF 输入法 DLL、后台服务、设置程序
cargo build --release -p qingjian-windows-server -p qingjian-windows-tsf -p qingjian-windows-settings
# 另编一份 32 位 TSF DLL（供 32 位进程输入）
cargo build --release -p qingjian-windows-tsf --target i686-pc-windows-msvc
# 打正式安装包（需要 Inno Setup）
apps/windows/installer/build.ps1
```

**macOS / Linux**：`cargo build --release` 后按[上游安装说明](https://qingjian.app/docs)部署对应输入法框架。

## 下载与使用

- **macOS / Windows**：安装后选中青简，在「设置 → 通用」选择想学习的语言即可开始输入；
- **Linux**：Fcitx5 版本，需手动启动后台服务（详见[上游安装说明](https://qingjian.app/docs/getting-started/linux)）；
- **命令模式开箱即用**：默认 Auto 模式，直接输入 `fas`、`adb`、`edl`、`ssh` 等前缀即可体验；想关掉或强制开启，用快捷键在 Auto / Manual / Off 间切换。

## 隐私与数据

拼音转换、词库查询、本地模型与输入统计全部在设备本地完成，不需要账号，不上传；可选云联想默认关闭，开启后数据直连你自行填写的 AI 服务商，不经过任何中转服务器。

## 常见问题

**命令模式会影响拼音输入吗？**
不会。默认 Auto 模式只在「输入纯命令字符且前缀命中命令库」时插入命令候选，且命令候选永远排在候选区最前；未命中的字母串自动回落为普通拼音候选，中文输入毫无感知。

**怎么关掉命令模式？**
用快捷键在 Auto → Manual → Off 间循环切换，或在设置页「命令」分类里直接关闭总开关，也可以按分类单独启停。

**命令模式是上游的功能吗？**
不是。命令模式、命令模块与随包命令库均为本仓库新增，上游青简不包含。

**想加自己的高频命令怎么办？**
编辑 `assets/commands/custom.tsv` 加一行即可（每行「命令 Tab 分类 Tab 说明」），或在设置页「命令」分类导入自己的 tsv；重新打包（或热重载）后生效。

**上游更新怎么同步？**
仓库以 `upstream` 指向上游 `qingjian-team/qingjian`，可直接 fetch 合并；命令模块独立在 `crates/qingjian-core/src/command/`，与上游改动冲突面很小。

## 反馈与参与

- 反馈问题或建议请到 [Issues](https://github.com/zh0ang/qingjian/issues)；
- 想了解命令模式实现：命令模块见 `crates/qingjian-core/src/command/`，命令库见 `assets/commands/`；
- 上游使用文档：[qingjian.app/docs](https://qingjian.app/docs)。
*（内容由AI生成，仅供参考）*
