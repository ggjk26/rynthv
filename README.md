# RynthV

<p align="center">
  <img src="assets/branding/app-icon.png" alt="RynthV 图标" width="96" height="96">
</p>

使用 Rust + egui 开发的歌声合成引擎。

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

## 声库与配置

首次使用需要在设置中指定声库目录。**普通话（常用字、拼音）、日语（假名、罗马音）和英语（CMUdict）已有内置发音表**，不再强制要求指定外部 `clf-data` 路径。外部词典目录仍可手动指定，供其他语言使用，或覆盖内置资源。工程中的自定义发音词典和音符显式音素仍优先于自动推导。

普通话内置常用字读音取自 `mozillazg/pinyin-data` 的《通用规范汉字表》数据（MIT 许可，参见 `licenses/pinyin-data-MIT.txt`）；部分多音词可在工程词典里指定准确读音。英文词典来自 CMU Pronouncing Dictionary，保留未编号的主读音，许可证见 licenses/cmudict-license.txt。日语内置资源支持常见假名及罗马音，但**不包含汉字自动注音**，需要外部 EDICT 或显式音素。内置音素映射尚需用实际声库核对，遇到未覆盖音素会报错而非静默替代。

声库列表现在能识别 SV Flat 的纯文本 NOFS-JSON，也支持识别相邻 config.json + info.nofs 的安装清单。设置中可单独选择 Flat 声库目录、Flat 模型目录（后者仅供依赖扫描）。目前可以读取元数据、唱法、部分 Float32 音色/音高/时长向量与模型标识，但模型文件存在并不代表引擎支持推理。

**重要：SV Flat 目前只有只读识别与诊断功能，不能合成/试听/导出。** .sfpk 安装包解包和 Flat 原生合成模型加载尚未实现，识别到的 Flat 声库会明确显示为不可用，不会误当作 SV1 声库加载。

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
