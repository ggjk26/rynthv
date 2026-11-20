//! Inspect Flat package metadata without decoding or loading protected models.
use rynthv::synthesis::voice::{VoiceCatalog, flat::{FlatManifest, FlatVoice}};
use serde_json::json;
use std::{env, path::{Path, PathBuf}, process::ExitCode};

fn main() -> ExitCode {
    match inspect() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn inspect() -> Result<(), String> {
    let mut path = None;
    let mut models = None;
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        let argument_text = argument.to_string_lossy();
        if argument_text == "--help" || argument_text == "-h" {
            println!("用法：rynthv-flat-inspect <NOFS-JSON|info.nofs|config.json|目录> [--models 模型目录]");
            println!("只读取元数据，不执行 Flat 歌声合成。输出 JSON 诊断报告。");
            return Ok(());
        }
        if argument_text == "--models" {
            models = Some(PathBuf::from(
                args.next().ok_or("--models 需要一个路径参数")?
            ));
        } else if argument_text.starts_with('-') {
            return Err(format!("未知参数：{argument_text}"));
        } else if path.is_none() {
            path = Some(PathBuf::from(argument));
        } else {
            return Err("只能指定一个声库路径".to_owned());
        }
    }
    let path: PathBuf = path.ok_or(
        "请提供 Flat NOFS-JSON、info.nofs、config.json 或声库目录"
    )?;
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
    } else if file_name_is(&path, "info.nofs")
        && path.parent().is_some_and(|dir| dir.join("config.json").is_file())
        && !is_json(&path)
    {
        json!({
            "kind": "flat-package-manifest",
            "manifest": FlatManifest::open(&path.parent().unwrap().join("config.json"))
                .map_err(|error| error.to_string())?,
        })
    } else if is_json(&path) {
        json!({
            "kind": "flat-nofs-json",
            "voice": FlatVoice::open(&path, models.as_deref())
                .map_err(|error| error.to_string())?,
        })
    } else {
        return Err("不支持的声库格式：非 Flat JSON 且没有 config.json 清单".to_owned());
    };
    let output = serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn file_name_is(path: &Path, name: &str) -> bool {
    path.file_name().is_some_and(|file| file.eq_ignore_ascii_case(name))
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
    bytes.iter().copied().find(|byte| !byte.is_ascii_whitespace()) == Some(b'{')
}
