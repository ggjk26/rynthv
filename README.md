# RynthV

<p align="center">
  <img src="assets/branding/app-icon.png" alt="RynthV 图标" width="96" height="96">
</p>

使用 Rust + egui 开发的歌声合成引擎。

## 旧工程兼容性（开发分支）

导入时不再因为 SVP 的数字版本（例如 113）与本项目默认保存版本不一致就拒绝打开；仍会校验时间轴、音符、引用和工程结构，无法安全解析的字段不会被悄悄忽略。旧工程可缺省部分音符、音轨和导出配置字段。导入时会去除旧 SV1 的 `dF0Vbr` / `dF0VbrEnv`，保留当前支持的 `dF0VbrMod`。

支持将 `br` 歌词映射至当前声库明确支持的同名呼吸音素；若声库没有该音素，仍走原有词典流程并报告不支持，避免伪造音素。混音导出文件名改为 `_mixdown.wav`。

这些改动不等于已完成 SVP 全字段兼容，也不等于已实现气声分离导出；后者仍需要真实的音频分离渲染后端。

## GitHub Actions 预构建

[Prebuild RynthV](https://github.com/ggjk26/rynthv/actions/workflows/prebuild.yml) 在 `master` 提交、Pull Request 和手动触发时构建 **Windows x64** 与 **Linux x64**。通过检查后，到对应工作流运行页面的 **Artifacts** 下载 `RynthV-Windows-x64` 或 `RynthV-Linux-x64`。压缩包包含图形编辑器、`rynthv-synth` 和 `rynthv-flat-inspect` 命令行工具及 README / LICENSE；不含商业声库或运行时模型。

推送 `v*` Git 标签时，工作流在双平台构建成功后发布 GitHub Release，并附上 ZIP / TAR.GZ 预构建包。PR 分支的构建产物不是正式发布版。工作流基于托管 Windows / Ubuntu runner，执行可能产生 GitHub Actions 配额或账单消耗；如果仓库禁用付费 runner，Windows 构建会无法启动。注意 Linux 构建仍依赖系统图形和音频运行库，并非完全静态可移植包。

## 构建与运行

需要 Rust 1.99.0，版本由 `rust-toolchain.toml` 指定。

### Windows

需要 MSVC、Windows SDK 和支持 DX12 的显卡。

```powershell
cargo run --release
```

### Linux

支持 X11 / Wayland，使用 Vulkan 或 OpenGL。Ubuntu 24.04 构建示例：

```sh
sudo apt install build-essential pkg-config libasound2-dev libwayland-dev libxkbcommon-dev \
  fontconfig fonts-noto-cjk libvulkan1 libegl1 libgl1 xdg-desktop-portal xdg-desktop-portal-gtk
cargo build --release --locked --bins
./target/release/rynthv
```

需要可用的图形驱动、ALSA 音频设备和桌面对应的 XDG 门户后端（KDE 使用 `xdg-desktop-portal-kde`）。

## 多语言界面（开发中）

在「设置 → 常规 → 界面语言」切换简体中文、English、日本語，选择会随界面偏好保存，旧配置文件在缺少 language 字段时仍使用简体中文。当前已完成设置导航、常规设置和部分声库管理界面翻译；其他编辑器工具、错误消息和声库详情仍在逐步本地化。声库自身支持的歌唱语言与**界面语言**是两项独立设置。

## 声库与配置

首次使用需要在设置中指定声库目录。**普通话（常用字、拼音）、日语（假名、罗马音）和英语（CMUdict）已有内置发音表**，不再强制要求指定外部 `clf-data` 路径。外部词典目录仍可手动指定，供其他语言使用，或覆盖内置资源。工程中的自定义发音词典和音符显式音素仍优先于自动推导。

普通话内置常用字读音取自 `mozillazg/pinyin-data` 的《通用规范汉字表》数据（MIT 许可，参见 `licenses/pinyin-data-MIT.txt`）；部分多音词可在工程词典里指定准确读音。英文词典来自 CMU Pronouncing Dictionary，保留未编号的主读音，许可证见 licenses/cmudict-license.txt。日语内置资源支持常见假名及罗马音，但**不包含汉字自动注音**，需要外部 EDICT 或显式音素。内置音素映射尚需用实际声库核对，遇到未覆盖音素会报错而非静默替代。

声库列表现在能识别 SV Flat 的纯文本 NOFS-JSON，也支持识别相邻 config.json + info.nofs 的安装清单。设置中可单独选择 Flat 声库目录、Flat 模型目录（后者仅供依赖扫描）。对相邻 config.json + info.nofs 清单，还会检查对应依赖模型的文件是否存在（不读取或解密模型内容）；非法模型路径不会被拼接访问。目前可以读取元数据、唱法、部分 Float32 音色/音高/时长向量与模型标识，但模型文件存在并不代表引擎支持推理。

**重要：SV Flat 目前只有只读识别与诊断功能，不能合成/试听/导出。** .sfpk 安装包解包和 Flat 原生合成模型加载尚未实现，识别到的 Flat 声库会明确显示为不可用，不会误当作 SV1 声库加载。

### 本地编译时内嵌 SV2 Duration（可选）

如果拥有相应模型的使用与再分发许可，可以在**自己的电脑**编译时通过
`RYNTHV_DURATION_DNNI` 指定一个 DNNI v3 文件。构建脚本只将该文件嵌入
**本地构建的可执行文件**，GitHub 仓库与默认 Actions 产物不会包含模型权重。

Windows PowerShell 示例：

```powershell
$env:RYNTHV_DURATION_DNNI = "D:\\MyModels\\duration.dnni"
cargo build --release --locked --bins
Remove-Item Env:RYNTHV_DURATION_DNNI
```

Linux 示例：

```sh
RYNTHV_DURATION_DNNI=/home/me/models/duration.dnni cargo build --release --locked --bins
```

运行时只会在所选 SV2 声库**没有自己的 Duration 阶段**时考虑使用内嵌模型。
它必须通过现有 DNNI v3 格式校验、资源完整性校验，并包含 **SV2 Duration**
资源标识；否则会报告不兼容，绝不会用其他类型的模型冒充时长模型。
有声库自带的 Duration 时优先使用声库自己的模型。

**SV Flat 的 `timing_model` 不等同于已验证兼容的 SV2 Duration 阶段。**
例如 `7b0ea690ada94b4484b50d9d64a21cae.dnni` 是 Flat 的通用
Timing Model；即使可以本地编译内嵌，也不保证能通过 RynthV 的 SV2 Duration
校验，更不能据此认为 SV Flat 原生合成已经实现。
`dcee89442f69984189a5b2aedbf9f090` 为 Flat F0，
`2c233d8e9b19f1f4dc0276ba3a5542c1` 为 Flat 基础模型，
均不应当作为 Duration 注入。

### SV Flat 命令行诊断

命令可用于检查原始 JSON NOFS、解包后配套 config.json 的 info.nofs，以及声库目录：

```powershell
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatVoice\voice.nofs" --models "D:\FlatModels"
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatVoice\info.nofs"
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatDatabases"
```

此命令不会解密模型或生成音频。用于模型格式适配前的元数据与资源诊断。

程序仍不自动探测模型目录，不附带商业声库或用户模型上下文。

| 声库类型 | Windows | Linux |
| --- | --- | --- |
| 一代声库 | 支持 | 支持 |
| 二代声库 | 需要导入模型上下文 | 需要导入模型上下文 |
| SV Flat（NOFS-JSON / 清单） | 只读诊断，尚不可合成 | 只读诊断，尚不可合成 |

配置目录：Windows 为 `%LOCALAPPDATA%/RynthV`，Linux 为 `$XDG_CONFIG_HOME/RynthV`（未设置时为 `$HOME/.config/RynthV`）。模型上下文在 Windows 保存在 `%APPDATA%/RynthV/context.json`，Linux 保存在上述配置目录的 `context.json`。

命令行合成与诊断工具的用法可通过 `cargo run --release --locked --bin rynthv-synth -- --help` 查看。
