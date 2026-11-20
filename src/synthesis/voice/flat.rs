//! Flat voicebank metadata and finite embedding vectors.
//!
//! Does not decrypt, synthesize, or write protected model resources. A Flat
//! voicebank is metadata plus references to separately installed base models.
use super::{Result, VoiceError};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

const MAX_JSON_BYTES: u64 = 1024 * 1024;
const MAX_STYLES: usize = 256;

#[derive(Clone, Debug, Serialize)]
pub struct FlatModelReference {
    pub role: &'static str,
    pub id: String,
    pub installed: bool,
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FlatStyle {
    pub name: String,
    pub vector: Vec<f32>,
    pub extra: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FlatVoice {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub language: String,
    pub supported_languages: Vec<String>,
    pub phoneset: Option<String>,
    pub models: Vec<FlatModelReference>,
    pub styles: Vec<FlatStyle>,
    pub pitch: Option<Vec<f32>>,
    pub timing: Option<Vec<f32>>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FlatManifest {
    pub name: String,
    pub version: String,
    pub model_ids: Vec<String>,
    pub model_types: Vec<String>,
    pub phonesets: Vec<String>,
    pub notes: Vec<String>,
}

impl FlatVoice {
    /// Only parses a self-contained text NOFS-JSON file. Model references are
    /// inspectable without reading their contents.
    pub fn open(path: &Path, model_root: Option<&Path>) -> Result<Self> {
        let value = read_json(path)?;
        let object = value.as_object().ok_or_else(|| {
            VoiceError::new("Flat NOFS-JSON 顶层必须是对象")
        })?;
        let mut notes = Vec::new();
        let mut models = Vec::new();
        for (key, role, folder) in [
            ("base_model", "base_model", "acoustic"),
            ("sing_model", "sing_model", "pitch"),
            ("timing_model", "timing_model", "duration"),
            ("f0_model", "f0_model", "f0"),
        ] {
            let Some(id) = optional_text(object, key)? else {
                notes.push(format!("{key} 未指定；需要根据具体 Flat 版本确认默认值"));
                continue;
            };
            let id = verify_model_id(id, key)?;
            let path = model_root.and_then(|root| locate_model(root, &id, folder));
            models.push(FlatModelReference {
                role, id, installed: path.is_some(), path,
            });
        }
        if let Some(root) = model_root {
            if !root.is_dir() {
                notes.push(format!("Flat 模型目录不存在：{}", root.display()));
            } else {
                for model in &models {
                    if !model.installed {
                        notes.push(format!("未定位到 {} 模型 {}（仅检查已配置目录）", model.role, model.id));
                    }
                }
            }
        } else {
            notes.push("尚未配置 Flat 模型目录，未进行模型文件检查".to_owned());
        }

        let styles = match object.get("styles") {
            None | Some(Value::Null) => {
                notes.push("未声明 styles 音色向量".to_owned());
                Vec::new()
            }
            Some(Value::Array(items)) if items.len() <= MAX_STYLES => items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let item = item.as_object().ok_or_else(|| {
                        VoiceError::new(format!("styles[{index}] 必须是对象"))
                    })?;
                    let name = required_text(item, "name")?.to_owned();
                    if name.trim().is_empty() {
                        return Err(VoiceError::new(format!("styles[{index}] 名称不能为空")));
                    }
                    let vector = parse_hex_vector(
                        required_text(item, "data")?,
                        32, &format!("styles[{index}].data")
                    )?;
                    let extra = match item.get("extra") {
                        None | Some(Value::Null) => None,
                        Some(number) => {
                            let value = number.as_f64()
                                .filter(|value| value.is_finite() && value.abs() <= 1_000_000.0)
                                .ok_or_else(|| VoiceError::new(format!(
                                    "styles[{index}].extra 必须是有限的数值"
                                )))?;
                            Some(value as f32)
                        }
                    };
                    Ok(FlatStyle { name, vector, extra })
                })
                .collect::<Result<Vec<_>>>()?,
            Some(Value::Array(_)) => return Err(VoiceError::new("Flat styles 超过 256 项上限")),
            Some(_) => return Err(VoiceError::new("Flat styles 必须是数组")),
        };
        let pitch = optional_vector(object, "pitch", 32)?;
        let timing = optional_vector(object, "timing", 128)?;
        let language = optional_text(object, "language")?.unwrap_or_default().to_owned();
        let mut supported_languages = text_array(object, "support_languages")?;
        if !language.is_empty() && !supported_languages.iter().any(|s| s == &language) {
            supported_languages.insert(0, language.clone());
        }
        Ok(Self {
            name: optional_text(object, "name")?.unwrap_or("SV Flat Voice").to_owned(),
            version: version(object)?,
            vendor: optional_text(object, "vendor")?.unwrap_or_default().to_owned(),
            language,
            supported_languages,
            phoneset: optional_text(object, "phoneset")?.map(str::to_owned),
            models, styles, pitch, timing, notes,
        })
    }
}

impl FlatManifest {
    /// Some SFPK distributions install a config.json alongside an opaque
    /// info.nofs. This sidecar is descriptive: it cannot decode that binary.
    pub fn open(path: &Path) -> Result<Self> {
        let value = read_json(path)?;
        let object = value.as_object()
            .ok_or_else(|| VoiceError::new("Flat config.json 顶层必须是对象"))?;
        let model_ids = text_array(object, "models")?;
        let model_types = text_array(object, "model_types")?;
        let phonesets = text_array(object, "phonesets")?;
        let mut notes = Vec::new();
        if model_ids.len() != model_types.len() {
            notes.push(format!(
                "config.json 的 models ({}) 与 model_types ({}) 数量不一致",
                model_ids.len(), model_types.len()
            ));
        }
        if model_ids.is_empty() {
            notes.push("config.json 未内嵌 Flat 模型清单".to_owned());
        }
        notes.push("info.nofs 是未解析的二进制资源；仅解析相邻 config.json".to_owned());
        Ok(Self {
            name: optional_text(object, "name")?.unwrap_or("SV Flat Package").to_owned(),
            version: version(object)?,
            model_ids,
            model_types,
            phonesets,
            notes,
        })
    }
}

/// 256 hexadecimal characters correspond to 32 little-endian float32 values.
/// Timing uses 1024 hex characters, or 128 float32 values.
pub fn parse_hex_vector(text: &str, count: usize, field: &str) -> Result<Vec<f32>> {
    if text.len() != count * 8 {
        return Err(VoiceError::new(format!(
            "{field} 长度应为 {} 位十六进制字符，实际 {}",
            count * 8, text.len()
        )));
    }
    let mut result = Vec::with_capacity(count);
    for (index, word) in text.as_bytes().chunks_exact(8).enumerate() {
        let mut bytes = [0u8; 4];
        for (pair, target) in word.chunks_exact(2).zip(&mut bytes) {
            let hex = std::str::from_utf8(pair)
                .map_err(|_| VoiceError::new(format!("{field} 含非 ASCII 字符")))?;
            *target = u8::from_str_radix(hex, 16)
                .map_err(|_| VoiceError::new(format!("{field} 包含无效十六进制字符")))?;
        }
        let value = f32::from_le_bytes(bytes);
        if !value.is_finite() {
            return Err(VoiceError::new(format!("{field}[{index}] 不是有限浮点数")));
        }
        result.push(value);
    }
    Ok(result)
}

fn optional_vector(
    object: &Map<String, Value>,
    key: &str,
    count: usize,
) -> Result<Option<Vec<f32>>> {
    optional_text(object, key)?
        .map(|value| parse_hex_vector(value, count, key))
        .transpose()
}

fn optional_text<'a>(object: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value)),
        Some(Value::String(_)) => Ok(None),
        Some(_) => Err(VoiceError::new(format!("Flat {key} 必须是字符串"))),
    }
}

fn required_text<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    optional_text(object, key)?.ok_or_else(|| VoiceError::new(format!(
        "Flat 元数据缺少必需字段 {key}"
    )))
}

fn text_array(object: &Map<String, Value>, key: &str) -> Result<Vec<String>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(values)) if values.len() <= 1024 => values.iter()
            .map(|value| value.as_str()
                .filter(|text| !text.is_empty() && text.len() < 4096)
                .map(str::to_owned)
                .ok_or_else(|| VoiceError::new(format!("{key} 数组含无效文本"))))
            .collect(),
        Some(Value::Array(_)) => Err(VoiceError::new(format!("{key} 数组过长"))),
        _ => Err(VoiceError::new(format!("{key} 必须是数组"))),
    }
}

fn version(object: &Map<String, Value>) -> Result<String> {
    match object.get("version") {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Number(number)) if number.is_u64() => Ok(number.to_string()),
        _ => Err(VoiceError::new("Flat version 必须是文本或非负整数")),
    }
}

fn verify_model_id(text: &str, field: &str) -> Result<String> {
    if text.len() != 32 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(VoiceError::new(format!("{field} 必须是 32 位十六进制模型标识")));
    }
    Ok(text.to_ascii_lowercase())
}

fn locate_model(root: &Path, id: &str, preferred_folder: &str) -> Option<PathBuf> {
    // The Flat field name is not a guaranteed DNNI stage role. Search common
    // install layouts without assuming which executable stage consumes it.
    // The validated 32-hex ID cannot escape the configured directory.
    let folders = [
        preferred_folder, "acoustic", "duration", "pitch",
        "f0", "vocoder", "sing", "timing", "base_model", "models",
    ];
    let direct = root.join(format!("{id}.dnni"));
    if direct.is_file() {
        return Some(direct);
    }
    for folder in folders {
        let parent = root.join(folder);
        for name in [format!("{id}.dnni"), id.to_owned()] {
            let candidate = parent.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn read_json(path: &Path) -> Result<Value> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_JSON_BYTES {
        return Err(VoiceError::new("Flat JSON 必须是小于 1 MiB 的普通文件"));
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(MAX_JSON_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err(VoiceError::new("Flat JSON 超过读取限制"));
    }
    let content = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    serde_json::from_slice(content)
        .map_err(|error| VoiceError::new(format!("Flat JSON 无法解析：{error}")))
}
