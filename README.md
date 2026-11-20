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

首次使用需要在设置中指定声库目录。**普通话（常用字、拼音）及日语（假名、罗马音）已有内置发音表**，不再强制要求指定外部 \`clf-data\` 路径。外部词典目录仍可手动指定，供其他语言使用或覆盖内置资源。工程中的自定义发音词典和音符显式音素仍优先于自动推导。

普通话内置常用字读音取自 \`mozillazg/pinyin-data\` 的《通用规范汉字表》数据（MIT 许可，参见 \`licenses/pinyin-data-MIT.txt\`）；部分多音词可在工程词典里指定准确读音。日语内置资源支持常见假名及罗马音，但**不包含汉字自动注音**，需要外部 EDICT 或显式音素。内置音素映射尚需用实际声库核对，遇到未覆盖音素会报错而非静默替代。

声库列表现在能**识别 SV Flat 的纯文本 NOFS-JSON**，展示名称、厂商、语言、版本和模型引用缺失情况。Flat 声库依赖独立的 \`base_model\` / \`sing_model\` / \`timing_model\` / \`f0_model\` 等资源；当前仅完成元数据识别与检查，**尚不能使用 Flat 声库合成或试听**。Flat 的 \`.sfpk\` 安装包解包及模型推理仍需后续适配。为防误用，这些声库在列表中标为“不可用”，且不会按 SV1 NOFS 解析。

程序仍不自动探测模型目录，不附带商业声库或用户模型上下文。

| 声库类型 | Windows | Linux |
| --- | --- | --- |
| 一代声库 | 支持 | 支持 |
| 二代声库 | 需要导入模型上下文 | 需要导入模型上下文 |
| SV Flat（NOFS-JSON） | 仅元数据识别，尚不可合成 | 仅元数据识别，尚不可合成 |

配置目录：Windows 为 `%LOCALAPPDATA%/RynthV`，Linux 为 `$XDG_CONFIG_HOME/RynthV`（未设置时为 `$HOME/.config/RynthV`）。模型上下文在 Windows 保存在 `%APPDATA%/RynthV/context.json`，Linux 保存在上述配置目录的 `context.json`。

命令行合成与诊断工具的用法可通过 `cargo run --release --locked --bin rynthv-synth -- --help` 查看。
