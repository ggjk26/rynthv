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

声库列表现在能识别 SV Flat 的纯文本 NOFS-JSON，也支持识别相邻 config.json + info.nofs 的安装清单。设置中可单独选择 Flat 声库目录、Flat 模型目录（后者仅供依赖扫描）。对相邻 config.json + info.nofs 清单，还会检查对应依赖模型的文件是否存在（不读取或解密模型内容）；非法模型路径不会被拼接访问。目前可以读取元数据、唱法、部分 Float32 音色/音高/时长向量与模型标识，但模型文件存在并不代表引擎支持推理。

**SV Flat 整曲合成、试听和音频导出仍待实现。** .sfpk 安装包解包和 Flat 原生合成模型加载尚未实现，识别到的 Flat 声库会明确显示为不可用，不会误当作 SV1 声库加载。

### Flat 合成兼容性预检（开发阶段）

已经增加可从「声库管理 → 选中 Flat 歌手 → 验证已安装的 Flat DNNI 模型」启动的**后台检查**。它会逐一打开已定位的模型，检测当前读取器支持的 DNNI v3 容器、验证索引与资源摘要，并对比是否出现已知 SV2 阶段资源标识。结果可复制为 JSON 诊断报告。**这只是模型解析和原生内核兼容性检查，不会将 Flat 声库标记为可合成。**

命令行示例（Windows PowerShell）：

```powershell
.\rynthv-flat-inspect.exe "C:\Users\admin\Documents\OPSV\Dreamtonics\Synthesizer V Studio\databases\<声库目录>\info.200.nofs" --models "C:\Users\admin\Documents\OPSV\Dreamtonics\Synthesizer V Studio\models" --probe-models
```

如果 Duration 文件放在独立目录，也可以追加：

```powershell
--timing "C:\path\to\7b0ea690ada94b4484b50d9d64a21cae.dnni"
```

工具仅在加上 `--probe-models` 时读取并验证大型模型文件。为了保护 UI 响应速度，声库列表的快速扫描依旧只检查是否能定位文件。预检同时输出未经合成使用的 Flat 条件输入：32 维歌声唱法向量、32 维 F0 风格向量、128 维 Duration 风格向量，维度互不混用。

如果 `UnsupportedFormat` 或 `InvalidNativeContainer` 出现，只能说明**当前 RynthV 读取器**不兼容该模型容器或校验参数，不能据此断言用户的模型文件损坏。下一阶段仍需要针对这些模型实现专用图计算、音素字典映射、声学条件、Pitch/F0、Duration 和 Vocoder 协议，验证成功后才会开放 Flat 原生播放和 WAV 导出。

### Flat v200 NOFS 与外部时长模型

RynthV 可以只读解析 Kasane Teto AI 2 v200 一类 **JSON 格式的 `info.200.nofs`**。该版本使用 `acoustic`、`vocoder`、`duration`、`f0` 四个模型引用键；旧版的 `base_model`、`sing_model`、`timing_model`、`f0_model` 也会识别。解析器同时支持十六进制小端 Float32 编码的 `styles[].extra`、32 维 `f0_style`、128 维 `duration_style`。识别上述字段与模型引用**不代表 Flat 原生推理可用**。

「**设置 → 声库目录**」现在除了 SV1、SV2、Flat 的数据库目录，还提供两种独立的时长模型文件选择：
- **SV2 Duration DNNI**：选择 DNNI v3 文件；声库缺少自身 Duration 时可作为共享模型加载，后台验证资源索引、SV2 Duration 角色和完整性。无效文件会在声库列表显示不兼容，不能替换其他模型。
- **Flat Timing Model DNNI（仅检查）**：手动选择与 Flat NOFS 里 `duration` / `timing_model` 标识**文件名一致**的 `.dnni`，例如声库声明 `7b0ea690ada94b4484b50d9d64a21cae` 时，文件名应为 `7b0ea690ada94b4484b50d9d64a21cae.dnni`。此设置只用于 Flat 依赖定位/诊断，不会注入 SV2 Duration 推理。

在 Windows 点击「**检测并应用本机 SV1 / SV2 / Flat 声库目录**」时，也会检查 `Documents/OPSV/Dreamtonics/Synthesizer V Studio/models` 是否存在并作为 Flat 共享模型目录；已有的手动目录会在默认路径不存在时保留。

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

### Flat F0 单阶段实验

`rynthv-flat-inspect --pitch-score` 可用 NOFS 的 `f0_style` 运行可读的内联 DDS F0 计算图。它使用声库声明的 F0 模型 ID 定位文件，要求模型为 DNNI v1/v2，并核对 `phonesets/*-phones.data` 的音素映射与图内字典一致。输出是带起始时间和采样步长的 MIDI 音高曲线 JSON。

```powershell
.\rynthv-flat-inspect.exe "D:\FlatVoice\info.200.nofs" --models "D:\ReadableModels" --phonesets "D:\FlatVoice\phonesets" --pitch-score "D:\score.json"
```

`score.json` 包含 `score`（采用 `examples/synthesis/mandarin-phrase.json` 的显式音素乐谱格式）和 `phone_seconds`（每条乐谱记录对应一个时长数组，每个音素对应一个秒数）。每条数组的总时长必须等于该记录的时长，分音节记录按 `parts` 均分。例如，时长为 0.5 秒的两个音素可填写 `[0.05, 0.45]`。

此实验使用 DDS 图的基础风格向量输入槽，尚未和 Flat 的运行结果逐帧比对。该命令要求已知音素时长，输出全句音高估计，包含静音时段。安装包中的 DNNI v3、Flat Duration、声学图和声码器仍待适配。

程序仍不自动探测模型目录，不附带商业声库或用户模型上下文。

| 声库类型 | Windows | Linux |
| --- | --- | --- |
| 一代声库 | 支持 | 支持 |
| 二代声库 | 需要导入模型上下文 | 需要导入模型上下文 |
| SV Flat（NOFS-JSON / 清单） | 只读诊断和 F0 单阶段实验 | 只读诊断和 F0 单阶段实验 |

配置目录：Windows 为 `%LOCALAPPDATA%/RynthV`，Linux 为 `$XDG_CONFIG_HOME/RynthV`（未设置时为 `$HOME/.config/RynthV`）。模型上下文在 Windows 保存在 `%APPDATA%/RynthV/context.json`，Linux 保存在上述配置目录的 `context.json`。

命令行合成与诊断工具的用法可通过 `cargo run --release --locked --bin rynthv-synth -- --help` 查看。
