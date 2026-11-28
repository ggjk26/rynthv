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

需要 MSVC、Windows SDK，以及支持 DirectX 12 或 Vulkan 的显卡与驱动。Windows Vulkan 后端需要可用的 Vulkan 驱动；在未提供 Vulkan 的机器上请选择 DX12。

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

### 界面 GPU 后端：DX12 / Vulkan

在「**设置 → 常规 → 图形与显示 → 界面图形后端**」选择图形 API：
- Windows：自动选择、**DirectX 12**、**Vulkan**。
- Linux：自动选择、**Vulkan**（自动模式也可使用 OpenGL）。
- 此选项影响的是 **egui/wgpu 界面渲染**，不改变歌声推理、音频声码器或训练计算设备。

图形设备在程序启动时创建，因此**需要关闭并重新启动 RynthV 才能应用新的后端**。设置页与「关于 RynthV」都会显示当前实际运行的 API；关于窗口还可复制 GPU、驱动、版本和运行状态诊断信息。

如果指定后端在当前系统不可用，运行以下命令临时覆盖设置（不修改已保存的偏好），然后在设置中选择可用的后端：

```powershell
.\target\release\rynthv.exe --graphics=auto
.\target\release\rynthv.exe --graphics=dx12
.\target\release\rynthv.exe --graphics=vulkan
```

Linux 可使用 `./target/release/rynthv --graphics=vulkan`。如需同时打开工程，把工程文件路径作为另一个参数传入即可。

手动恢复时，也可编辑配置目录中的 `preferences.json`，将 `graphics_backend` 设为 `"auto"`。如果强制选择的后端缺少 GPU 驱动，程序可能在创建窗口前失败；CLI 参数提供了恢复入口。

## 多语言界面（开发中）

在「设置 → 常规 → 界面语言」切换简体中文、English、日本語，选择会随界面偏好保存，旧配置文件在缺少 language 字段时仍使用简体中文。当前已完成设置导航、常规设置和部分声库管理界面翻译；其他编辑器工具、错误消息和声库详情仍在逐步本地化。声库自身支持的歌唱语言与**界面语言**是两项独立设置。

## 声库与配置

首次使用需要在设置中指定声库目录。**普通话（常用字、拼音）、日语（假名、罗马音）和英语（CMUdict）已有内置发音表**，不再强制要求指定外部 `clf-data` 路径。外部词典目录仍可手动指定，供其他语言使用，或覆盖内置资源。工程中的自定义发音词典和音符显式音素仍优先于自动推导。

普通话内置常用字读音取自 `mozillazg/pinyin-data` 的《通用规范汉字表》数据（MIT 许可，参见 `licenses/pinyin-data-MIT.txt`）；部分多音词可在工程词典里指定准确读音。英文词典来自 CMU Pronouncing Dictionary，保留未编号的主读音，许可证见 licenses/cmudict-license.txt。日语内置资源支持常见假名及罗马音，但**不包含汉字自动注音**，需要外部 EDICT 或显式音素。内置音素映射尚需用实际声库核对，遇到未覆盖音素会报错而非静默替代。

### SV Flat NOFS-JSON 合成

在「设置 → 声库目录」选择 Flat 声库目录和 Flat 模型目录。若安装布局为同一级目录下的 `databases` 和 `models`，可自动定位模型。选择包含 `acoustic`、`duration`、`f0` 和 `vocoder` 模型引用的文本 NOFS-JSON 声库即可使用试听和导出。可在 **Flat Timing DNNI** 单独选择与清单模型 ID 同名的时长文件。

支持内联 DDS DNNI v1/v2 和 Flat legacy DNNI v3。读取 legacy v3 时，在启动 RynthV 前将本地 32 字节密钥以 64 个十六进制字符设置到 `RYNTHV_FLAT_LEGACY_KEY`。Windows 上从已设置该变量的 PowerShell 启动 RynthV。模型和密钥由本机提供，仓库与默认构建产物均不附带。二进制 `info.nofs` 和 `.sfpk` 安装清单目前可用于依赖诊断。

RynthV 使用清单里的 128 维 `duration_style`、32 维 `f0_style` 和唱法音色向量。唱法权重复用 DDS 混合，`styles[].extra` 尚未参与合成。

```powershell
.\rynthv-synth.exe render-score --voice "D:\FlatVoice\info.200.nofs" --flat-models "D:\FlatModels" --input "D:\score.json" --wav "D:\render.wav"
```

`--input` 接受 `examples/synthesis/mandarin-phrase.json` 所示的显式音素乐谱格式。可用 `--flat-timing` 指定独立时长模型文件。

### Flat 模型预检

在「声库管理」选中 Flat 声库后，可在后台验证四个 DDS 阶段。声库列表扫描只检查依赖和文件头，完整模型检查在加载或手动预检时执行。

```powershell
.\rynthv-flat-inspect.exe "D:\FlatVoice\info.200.nofs" --models "D:\FlatModels" --probe-models
.\rynthv-flat-inspect.exe --probe-file duration "D:\models\timing.dnni"
```

预检支持用 `--timing` 指定独立时长文件。报告会区分 DDS 阶段装配、legacy 解码以及签名容器的头部、索引与资源校验结果。

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

### SV Flat 命令行诊断

命令可用于检查原始 JSON NOFS、解包后配套 config.json 的 info.nofs，以及声库目录：

```powershell
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatVoice\voice.nofs" --models "D:\FlatModels"
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatVoice\info.nofs"
cargo run --release --locked --bin rynthv-flat-inspect -- "D:\FlatDatabases"
```

以上命令读取声库元数据和模型引用。添加 `--probe-models` 可进一步检查模型。

### Flat F0 单阶段实验

`rynthv-flat-inspect --pitch-score` 可用 NOFS 的 `f0_style` 运行可读的内联 DDS F0 计算图。它使用声库声明的 F0 模型 ID 定位文件，支持 DNNI v1/v2 和已配置密钥的 legacy v3，并核对 `phonesets/*-phones.data` 的音素映射与图内字典一致。输出是带起始时间和采样步长的 MIDI 音高曲线 JSON。

```powershell
.\rynthv-flat-inspect.exe "D:\FlatVoice\info.200.nofs" --models "D:\ReadableModels" --phonesets "D:\FlatVoice\phonesets" --pitch-score "D:\score.json"
```

`score.json` 包含 `score`（采用 `examples/synthesis/mandarin-phrase.json` 的显式音素乐谱格式）和 `phone_seconds`（每条乐谱记录对应一个时长数组，每个音素对应一个秒数）。每条数组的总时长必须等于该记录的时长，分音节记录按 `parts` 均分。例如，时长为 0.5 秒的两个音素可填写 `[0.05, 0.45]`。

此实验使用 DDS 图的基础风格向量输入槽，尚未和 Flat 的运行结果逐帧比对。该命令要求已知音素时长，输出全句音高估计，包含静音时段。需要由时长模型推导时间并生成 WAV 时，使用 `rynthv-synth render-score`。

### Flat 声学阶段实验

`--acoustic-score` 读取和 `--pitch-score` 相同的乐谱，先预测 F0，再执行可读的内联 DDS 声学图。模型目录应同时包含 NOFS 声明的 F0 和声学模型。可用 `--style` 指定一个 NOFS 唱法，默认使用 `(base)`（若无此项则使用首个唱法）。

```powershell
.\rynthv-flat-inspect.exe "D:\FlatVoice\info.200.nofs" --models "D:\ReadableModels" --phonesets "D:\FlatVoice\phonesets" --acoustic-score "D:\score.json" --style "Mellow"
```

输出包括声学模型和 F0 模型的标识、唱法、帧长与逐帧排列的 72 通道声学参数。音素时长仍来自输入乐谱。实验将唱法向量接入声学图的风格槽，保留该图的固定条件向量。唱法的 `extra` 字段尚未接入，声学结果尚未与 SV Flat 逐帧比对。

| 声库类型 | Windows | Linux |
| --- | --- | --- |
| 一代声库 | 支持 | 支持 |
| 二代声库 | 需要导入模型上下文 | 需要导入模型上下文 |
| SV Flat（NOFS-JSON + DDS） | 支持合成 | 支持合成 |
| SV Flat（二进制 info.nofs / 安装清单） | 依赖诊断 | 依赖诊断 |

配置目录：Windows 为 `%LOCALAPPDATA%/RynthV`，Linux 为 `$XDG_CONFIG_HOME/RynthV`（未设置时为 `$HOME/.config/RynthV`）。模型上下文在 Windows 保存在 `%APPDATA%/RynthV/context.json`，Linux 保存在上述配置目录的 `context.json`。

命令行合成与诊断工具的用法可通过 `cargo run --release --locked --bin rynthv-synth -- --help` 查看。
