use super::{Runtime, events::WakeOnExit, mixer};
use crate::{
    model::{Extra, Project},
    synthesis::audio::{self, ChannelLayout, WaveFormat},
};
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportPart {
    Full,
    Primary,
    Secondary,
}

#[derive(Clone, Copy)]
enum AspirationMode {
    None,
    Separate,
    PrimaryOnly,
    SecondaryOnly,
}

pub struct ExportRequest {
    pub outputs: Vec<(PathBuf, Option<u64>, ExportPart)>,
    format: WaveFormat,
}

fn add_outputs(
    outputs: &mut Vec<(PathBuf, Option<u64>, ExportPart)>,
    destination: &Path,
    stem: &str,
    track: Option<u64>,
    mode: AspirationMode,
) {
    let base = destination.join(format!("{stem}.wav"));
    let asp = destination.join(format!("{stem}_asp.wav"));
    match mode {
        AspirationMode::None => outputs.push((base, track, ExportPart::Full)),
        AspirationMode::Separate => {
            outputs.push((base, track, ExportPart::Primary));
            outputs.push((asp, track, ExportPart::Secondary));
        }
        AspirationMode::PrimaryOnly => outputs.push((base, track, ExportPart::Primary)),
        AspirationMode::SecondaryOnly => outputs.push((asp, track, ExportPart::Secondary)),
    }
}

struct ExportOptions<'a> {
    directory: &'a Path,
    stem: &'a str,
    format: WaveFormat,
    mode: AspirationMode,
    mixdown: bool,
}

impl<'a> ExportOptions<'a> {
    fn read(config: &'a Extra, has_tracks: bool) -> Result<Self, String> {
        let text = |key: &str| {
            config
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        };
        let number = |key: &str, default| {
            config
                .get(key)
                .map_or(Some(default), serde_json::Value::as_u64)
        };
        let directory = Path::new(text("destination"));
        if directory.as_os_str().is_empty() {
            return Err("请选择目标文件夹".into());
        }
        let name = text("filename").trim();
        let stem = match name.rsplit_once('.') {
            Some((stem, extension)) if extension.eq_ignore_ascii_case("wav") => stem,
            _ => name,
        };
        if stem.is_empty()
            || stem.ends_with(['.', ' '])
            || reserved_name(stem)
            || stem
                .chars()
                .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        {
            return Err("请输入有效的文件名称".into());
        }
        let sample_rate = match number("sampleRate", 44100) {
            Some(rate @ (44100 | 48000 | 96000)) => rate as u32,
            _ => return Err("请选择有效的采样率".into()),
        };
        let bit_depth = match number("bitDepth", 16) {
            Some(depth @ (16 | 24 | 32)) => depth as u16,
            _ => return Err("请选择有效的位深度".into()),
        };
        let layout = match number("numChannels", 1) {
            Some(1) => ChannelLayout::Mono,
            Some(2) => ChannelLayout::Stereo,
            _ => return Err("请选择有效的声道数".into()),
        };
        let mode = match config
            .get("aspirationFormat")
            .map(serde_json::Value::as_str)
        {
            None | Some(Some("" | "noAspiration")) => AspirationMode::None,
            Some(Some("separateAspiration")) => AspirationMode::Separate,
            Some(Some("onlyMain")) => AspirationMode::PrimaryOnly,
            Some(Some("onlyAspiration")) => AspirationMode::SecondaryOnly,
            _ => return Err("请选择有效的气声输出模式".into()),
        };
        let mixdown = export_mixdown(config);
        if !mixdown && !has_tracks {
            return Err("请勾选混音导出或至少一条音轨".into());
        }
        crate::synthesis::mixing_options::validate_export_bypass(config)?;
        Ok(Self {
            directory,
            stem,
            format: WaveFormat {
                sample_rate,
                layout,
                bit_depth,
            },
            mode,
            mixdown,
        })
    }
}

fn export_mixdown(config: &Extra) -> bool {
    config
        .get("exportMixDown")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
}

impl ExportRequest {
    pub fn check_options(config: &Extra, has_tracks: bool) -> Result<(), String> {
        ExportOptions::read(config, has_tracks).map(|_| ())
    }

    pub fn from_project(project: &Project) -> Result<Self, String> {
        let enabled = |track: &crate::model::Track| {
            track
                .extra
                .get("renderEnabled")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
        };
        let options =
            ExportOptions::read(&project.render_config, project.tracks.iter().any(enabled))?;
        match std::fs::metadata(options.directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Err("导出目标路径不是文件夹".into()),
            Err(error) => {
                return Err(format!(
                    "无法访问目标文件夹 {}：{error}",
                    options.directory.display()
                ));
            }
        }
        crate::synthesis::mixing_options::validate_project_mixer(
            project.extra.get("projectMixer"),
        )?;
        for track in &project.tracks {
            crate::synthesis::mixing_options::validate_mixer(&track.mixer)
                .map_err(|error| format!("音轨 {}：{error}", track.name))?;
        }
        let mut outputs = Vec::new();
        if options.mixdown {
            add_outputs(
                &mut outputs,
                options.directory,
                &format!("{}_mixdown", options.stem),
                None,
                options.mode,
            );
        }
        for (index, track) in project
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| enabled(track))
        {
            let safe: String = track
                .name
                .chars()
                .map(|c| {
                    if c.is_control() || "<>:\"/\\|?*".contains(c) {
                        '_'
                    } else {
                        c
                    }
                })
                .collect();
            add_outputs(
                &mut outputs,
                options.directory,
                &format!(
                    "{}_{:02}_{}",
                    options.stem,
                    index + 1,
                    safe.trim_end_matches(['.', ' '])
                ),
                Some(track.id),
                options.mode,
            );
        }
        Ok(Self {
            outputs,
            format: options.format,
        })
    }
}

fn reserved_name(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (1..=9).any(|n| base == format!("COM{n}") || base == format!("LPT{n}"))
}

impl Runtime {
    pub fn export_ready(&self, revision: u64) -> bool {
        self.last_revision == Some(revision)
            && self.export.is_none()
            && self.recording_error.is_none()
            && self.recordings.pending.is_empty()
            // Missing/unresolvable singer assignments must not globally
            // block the finished vocal tracks or instrumental recordings.
            // Only active synthesis/loading still prevents an incomplete export.
            && !self.is_busy()
            && (!self.export_owners.is_empty() || !self.recordings.layers.is_empty())
    }

    pub fn export_project(&mut self, request: ExportRequest, revision: u64) {
        if !self.export_ready(revision) {
            self.export_result = Some(Err("请等待当前工程合成完成后再导出".into()));
            return;
        }
        let owners = self.export_owners.clone();
        let recordings = self.recordings.layers.clone();
        let solo = self.solo_tracks.clone();
        let repaint = self.repaint.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        self.export_result = None;
        match thread::Builder::new()
            .name("audio-export".into())
            .spawn(move || {
                let _wake = WakeOnExit(repaint);
                let sender = tx;
                let result = (|| {
                    // Validate all selected sources before writing the first file.
                    let mut sources = Vec::new();
                    for target in request.outputs.chunk_by(|a, b| a.1 == b.1) {
                        let track = target[0].1;
                        let stems = owners
                            .iter()
                            .filter(|owner| track.is_none_or(|id| owner.controls.track_id == id));
                        let backing = recordings
                            .iter()
                            .filter(|layer| track.is_none_or(|id| layer.track_id == id));
                        let mix = mixer::Mix::prepare(stems, &solo, backing)?;
                        for (path, _, part) in target {
                            if let Some(audio) = mix.render(*part)? {
                                sources.push((path, audio));
                            }
                        }
                    }
                    if sources.is_empty() {
                        return Err("所选音轨均没有可导出的音频。".into());
                    }
                    let mut written = Vec::new();
                    for (path, audio) in sources {
                        audio::save_wav_as(path, &audio, request.format)
                            .map_err(|e| format!("{e}（此前已完成 {} 个文件）", written.len()))?;
                        written.push(path.clone());
                    }
                    Ok(written)
                })();
                let _ = sender.send(result);
            }) {
            Ok(_) => self.export = Some(rx),
            Err(error) => self.export_result = Some(Err(format!("无法启动音频导出：{error}"))),
        }
    }
}
