//! Inspect Flat metadata and run experimental inline DDS inference.
use rynthv::synthesis::voice::{
    VoiceCatalog,
    flat::{FlatManifest, FlatTimedScore, FlatVoice},
};
use serde_json::json;
use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    match inspect() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScoreMode {
    Pitch,
    Acoustic,
}

fn inspect() -> Result<(), String> {
    let mut path = None;
    let mut models = None;
    let mut timing = None;
    let mut phonesets = None;
    let mut score_request = None;
    let mut style = None;
    let mut probe_models = false;
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        let argument_text = argument.to_string_lossy();
        if argument_text == "--help" || argument_text == "-h" {
            println!(
                "用法：rynthv-flat-inspect <NOFS-JSON|info.nofs|config.json|目录> [--models 模型目录] [--timing 时长模型.dnni] [--probe-models]"
            );
            println!(
                "默认只读取元数据。--probe-models 会校验本地 DNNI v3 容器索引及资源完整性，并检查可读的内联 F0 和声学图，可能需要较长时间和较多内存；不执行歌声合成。"
            );
            println!(
                "实验性 F0：rynthv-flat-inspect <NOFS-JSON> --models <目录> --phonesets <目录> --pitch-score <已标注时长的乐谱.json>"
            );
            println!(
                "实验性 F0 + 声学：使用 --acoustic-score <乐谱.json> [--style <NOFS 唱法名>] 代替 --pitch-score。"
            );
            println!("实验需要可读的 DNNI v1/v2 DDS 模型，输出音高或声学帧 JSON。");
            return Ok(());
        }
        if argument_text == "--models" {
            models = Some(PathBuf::from(
                args.next().ok_or("--models 需要一个路径参数")?,
            ));
        } else if argument_text == "--timing" {
            timing = Some(PathBuf::from(
                args.next().ok_or("--timing 需要一个 .dnni 文件路径")?,
            ));
        } else if argument_text == "--phonesets" {
            phonesets = Some(PathBuf::from(
                args.next().ok_or("--phonesets 需要一个目录参数")?,
            ));
        } else if matches!(argument_text.as_ref(), "--pitch-score" | "--acoustic-score") {
            if score_request.is_some() {
                return Err("只可指定一种乐谱推理模式".into());
            }
            let mode = if argument_text == "--pitch-score" {
                ScoreMode::Pitch
            } else {
                ScoreMode::Acoustic
            };
            let score = PathBuf::from(
                args.next()
                    .ok_or_else(|| format!("{argument_text} 需要一个 JSON 文件路径"))?,
            );
            score_request = Some((mode, score));
        } else if argument_text == "--style" {
            style = Some(
                args.next()
                    .and_then(|arg| arg.into_string().ok())
                    .ok_or("--style 需要一个唱法名")?,
            );
        } else if argument_text == "--probe-models" {
            probe_models = true;
        } else if argument_text.starts_with('-') {
            return Err(format!("未知参数：{argument_text}"));
        } else if path.is_none() {
            path = Some(PathBuf::from(argument));
        } else {
            return Err("只能指定一个声库路径".to_owned());
        }
    }
    let path: PathBuf = path.ok_or("请提供 Flat NOFS-JSON、info.nofs、config.json 或声库目录")?;
    if score_request.is_some() != phonesets.is_some() {
        return Err("乐谱推理模式需要同时指定 --phonesets".into());
    }
    if style.is_some() && !matches!(score_request, Some((ScoreMode::Acoustic, _))) {
        return Err("--style 仅用于 --acoustic-score".into());
    }
    if score_request.is_some() && (probe_models || timing.is_some()) {
        return Err("请将乐谱推理模式与 --probe-models / --timing 分开运行".into());
    }
    if score_request.is_some() && (file_name_is(&path, "config.json") || !is_json(&path)) {
        return Err("乐谱推理需要包含 f0_style 的文本 Flat NOFS".into());
    }
    if (probe_models || timing.is_some()) && (path.is_dir() || file_name_is(&path, "config.json")) {
        return Err(
            "模型预检需要一个具体的 Flat JSON NOFS 文件；请指定 info.200.nofs 等文件".into(),
        );
    }
    if timing.is_some() && !probe_models {
        return Err("--timing 需要同时指定 --probe-models".into());
    }
    if probe_models && !is_json(&path) {
        return Err("DNNI 模型预检只支持 Flat JSON NOFS；加密的 info.nofs 仅能通过 config.json 检查安装清单".into());
    }
    let result = if path.is_dir() {
        json!({
            "kind": "catalog",
            "catalog": VoiceCatalog::scan_with_models(&path, models.as_deref())
                .map_err(|error| error.to_string())?,
        })
    } else if file_name_is(&path, "config.json") {
        json!({
            "kind": "flat-package-manifest",
            "manifest": FlatManifest::open(&path).map_err(|error| error.to_string())?,
        })
    } else if file_name_is(&path, "info.nofs") && !is_json(&path) {
        json!({
            "kind": "flat-package-manifest",
            "manifest": FlatManifest::open(&path.parent().unwrap().join("config.json"))
                .map_err(|error| error.to_string())?,
        })
    } else if is_json(&path) {
        let voice = FlatVoice::open(&path, models.as_deref()).map_err(|error| error.to_string())?;
        if let Some(((mode, score), phonesets)) = score_request.zip(phonesets) {
            let input = std::fs::File::open(&score)
                .map_err(|error| format!("{}: {error}", score.display()))?;
            let input: FlatTimedScore = serde_json::from_reader(std::io::BufReader::new(input))
                .map_err(|error| format!("{}: {error}", score.display()))?;
            match mode {
                ScoreMode::Pitch => {
                    let pitch = voice
                        .predict_inline_f0(&input, &phonesets)
                        .map_err(|error| error.to_string())?;
                    json!({
                        "kind": "flat-experimental-inline-dds-f0",
                        "voice": voice.name,
                        "version": voice.version,
                        "pitch": pitch,
                    })
                }
                ScoreMode::Acoustic => {
                    let acoustic = voice
                        .predict_inline_acoustics(&input, &phonesets, style.as_deref())
                        .map_err(|error| error.to_string())?;
                    json!({
                        "kind": "flat-experimental-inline-dds-acoustics",
                        "voice": voice.name,
                        "version": voice.version,
                        "acoustic": acoustic,
                    })
                }
            }
        } else if probe_models {
            let preflight = voice.preflight(timing.as_deref());
            let conditioning = match voice.conditioning(None) {
                Ok(controls) => json!({
                    "status": "parsed_only",
                    "inputs": controls,
                }),
                Err(error) => json!({
                    "status": "unavailable",
                    "reason": error.to_string(),
                }),
            };
            json!({
                "kind": "flat-model-preflight",
                "preflight": preflight,
                "conditioning": conditioning,
                "voice": voice,
            })
        } else {
            json!({
                "kind": "flat-nofs-json",
                "voice": voice,
            })
        }
    } else {
        return Err("不支持的声库格式：非 Flat JSON 且没有 config.json 清单".to_owned());
    };
    let output = serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn file_name_is(path: &Path, name: &str) -> bool {
    path.file_name()
        .is_some_and(|file| file.eq_ignore_ascii_case(name))
}

fn is_json(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut prefix = [0u8; 128];
    let Ok(len) = file.read(&mut prefix) else {
        return false;
    };
    let bytes = &prefix[..len];
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(b'{')
}
