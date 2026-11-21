use crate::model::*;
use std::{collections::HashSet, fs, io::Write, path::Path};

mod identity;

pub fn load(path: &Path) -> Result<Project, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("工程超过 64 MiB 读取上限".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut project: Project =
        serde_json::from_slice(&bytes).map_err(|e| format!("SVP 解析失败：{e}"))?;
    normalize_legacy_project(&mut project);
    validate(&project)?;
    let base = std::path::absolute(path).map_err(|e| e.to_string())?;
    for track in &mut project.tracks {
        if let Some(clip) = track.instrumental.as_mut().and_then(|t| t.clip.as_mut())
            && clip.path.is_relative()
        {
            clip.path = base.parent().unwrap().join(&clip.path);
        }
    }
    project.tracks.sort_by_key(|t| t.disp_order);
    for group in project
        .library
        .iter_mut()
        .chain(project.tracks.iter_mut().map(|t| &mut t.main_group))
    {
        group.notes.sort_by_key(|n| n.onset);
    }
    Ok(project)
}

pub fn save(path: &Path, project: &Project) -> Result<(), String> {
    validate(project)?;
    let mut ordered = project.clone();
    let base = std::path::absolute(path).map_err(|e| e.to_string())?;
    for track in &mut ordered.tracks {
        if let Some(clip) = track.instrumental.as_mut().and_then(|t| t.clip.as_mut())
            && let Ok(relative) = clip.path.strip_prefix(base.parent().unwrap())
        {
            clip.path = relative.to_owned();
        }
    }
    for group in ordered
        .library
        .iter_mut()
        .chain(ordered.tracks.iter_mut().map(|t| &mut t.main_group))
    {
        group.notes.sort_by_key(|n| n.onset);
    }
    let bytes = serde_json::to_vec_pretty(&ordered).map_err(|e| e.to_string())?;
    atomic_write(path, &bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e| format!("写入 {} 失败：{e}", path.display()))
}

fn validate(p: &Project) -> Result<(), String> {
    let bad = |s: &str| Err(format!("工程校验失败：{s}"));
    identity::validate(p)?;
    if p.tracks.is_empty() {
        return bad("至少需要一条音轨");
    }
    for (name, dictionary) in &p.dictionaries {
        if name.trim().is_empty() || name != name.trim() {
            return bad("发音词典名称不能为空或包含首尾空白");
        }
        if dictionary.entries.iter().any(|(word, phones)| {
            word.trim().is_empty()
                || word != word.trim()
                || phones.split_whitespace().next().is_none()
        }) {
            return bad("发音词典的字词或音素无效");
        }
    }
    if p.time.tempo.first().is_none_or(|t| t.position != 0)
        || p.time.meter.first().is_none_or(|m| m.index != 0)
    {
        return bad("时间轴必须从 0 开始");
    }
    if !p.time.start_time_seconds.is_finite()
        || p.time
            .tempo
            .iter()
            .any(|t| !t.bpm.is_finite() || t.bpm <= 0.0 || t.position < 0)
        || p.time
            .tempo
            .windows(2)
            .any(|a| a[0].position >= a[1].position)
    {
        return bad("曲速必须为正数，标记按时间严格递增");
    }
    if p.time.meter.iter().any(|m| {
        m.index < 0
            || m.numerator == 0
            || m.numerator > 64
            || !m.denominator.is_power_of_two()
            || m.denominator > 64
    }) || p.time.meter.windows(2).any(|a| a[0].index >= a[1].index)
    {
        return bad("拍号或拍号位置无效");
    }
    let mut ids = HashSet::new();
    for g in p
        .library
        .iter()
        .chain(p.tracks.iter().map(|t| &t.main_group))
    {
        if g.uuid.is_empty() || !ids.insert(g.uuid.as_str()) {
            return bad("音符组 UUID 缺失或重复");
        }
        if g.notes.iter().any(|n| {
            n.duration <= 0
                || n.onset.checked_add(n.duration).is_none()
                || !(0..=127).contains(&n.pitch)
        }) {
            return bad("音符时间或 MIDI 音高无效");
        }
        for note in &g.notes {
            NotePerformance::from_note(note)
                .map_err(|error| format!("工程校验失败：音符 {}：{error}", note.lyrics))?;
            PhonemeAttributes::read(&note.attributes)
                .map_err(|error| format!("工程校验失败：音符 {}：{error}", note.lyrics))?;
            TakeSet::from_extra(&note.extra)
                .map_err(|error| format!("工程校验失败：音符 {}：{error}", note.lyrics))?;
        }
        for (name, curve) in g.parameters.iter().chain(&g.vocal_modes) {
            if name.trim().is_empty() {
                return bad("参数名称不能为空");
            }
            curve
                .validate()
                .map_err(|error| format!("工程校验失败：参数 {name}：{error}"))?;
        }
        for control in &g.pitch_controls {
            match control {
                PitchControl::Point(point) if !point.valid() => {
                    return bad("音高控制点的音高或标识无效");
                }
                PitchControl::Curve(curve) if !curve.valid() => {
                    return bad("手绘音高曲线必须包含至少两个按时间递增的有效控制点");
                }
                _ => {}
            }
        }
    }
    for track in &p.tracks {
        if track.audio_clip().is_some_and(|c| !c.valid()) {
            return bad("伴奏文件路径或时间范围无效");
        }
        if track.is_instrumental()
            && (!track.groups.is_empty() || !track.main_group.notes.is_empty())
        {
            return bad("伴奏轨不能包含音符组");
        }
        if track.main_ref.group_id != track.main_group.uuid {
            return bad("主组引用不匹配");
        }
        for r in track.references() {
            TakeSet::from_extra(&r.extra)
                .map_err(|error| format!("工程校验失败：音轨 {}：{error}", track.name))?;
            let Some(group) = p.group(&r.group_id) else {
                return bad("存在悬空音符组引用");
            };
            if r.blick_absolute_begin < 0
                || (r.blick_absolute_end != -1 && r.blick_absolute_end < r.blick_absolute_begin)
            {
                return bad("片段可见范围无效");
            }
            if group.notes.iter().any(|n| {
                n.onset.checked_add(r.blick_offset).is_none()
                    || (n.onset + n.duration).checked_add(r.blick_offset).is_none()
            }) {
                return bad("实例时间溢出");
            }
            if group.pitch_controls.iter().any(|control| match control {
                PitchControl::Point(point) => point.pos.checked_add(r.blick_offset).is_none(),
                PitchControl::Curve(curve) => curve
                    .absolute_points()
                    .any(|(time, _)| time.checked_add(r.blick_offset).is_none()),
                PitchControl::Unsupported(_) => false,
            }) {
                return bad("实例音高控制时间溢出");
            }
        }
    }
    Ok(())
}


/// Normalize obsolete SV1 controls on import only. Modern dF0VbrMod is a
/// supported per-note vibrato control and must not be erased.
fn normalize_legacy_project(project: &mut Project) {
    // Older SVP revisions can omit render settings entirely. Supply only
    // absent keys, leaving any explicitly configured export options intact.
    for (key, value) in Project::default().render_config {
        project.render_config.entry(key).or_insert(value);
    }
    for group in project.library.iter_mut()
        .chain(project.tracks.iter_mut().map(|track| &mut track.main_group))
    {
        for key in ["dF0Vbr", "dF0VbrEnv"] {
            group.parameters.remove(key);
            group.vocal_modes.remove(key);
        }
        for note in &mut group.notes {
            for key in ["dF0Vbr", "dF0VbrEnv"] {
                note.attributes.remove(key);
                note.extra.remove(key);
            }
        }
    }
}

#[cfg(test)]
mod legacy_compat_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_svp_113_and_missing_optional_fields() {
        let mut value = serde_json::to_value(Project::default()).unwrap();
        value["version"] = json!(113);
        let note = Note::new(0, QUARTER, 60);
        let mut note = serde_json::to_value(note).unwrap();
        note.as_object_mut().unwrap().remove("phonemes");
        note.as_object_mut().unwrap().remove("attributes");
        note.as_object_mut().unwrap().remove("musicalType");
        value["tracks"][0]["mainGroup"]["notes"] = json!([note]);
        value.as_object_mut().unwrap().remove("library");
        value.as_object_mut().unwrap().remove("renderConfig");
        value["time"].as_object_mut().unwrap().remove("startTimeSeconds");
        let project: Project = serde_json::from_value(value).unwrap();
        assert_eq!(project.version, 113);
        assert_eq!(project.tracks[0].main_group.notes[0].musical_type, "singing");
        validate(&project).unwrap();
    }

    #[test]
    fn old_vibrato_is_removed_but_supported_modulator_remains() {
        let mut project = Project::default();
        let group = &mut project.tracks[0].main_group;
        group.parameters.insert("dF0Vbr".into(), Automation::default());
        let mut note = Note::new(0, QUARTER, 60);
        note.attributes.insert("dF0Vbr".into(), json!(0.3));
        note.attributes.insert("dF0VbrMod".into(), json!(1.2));
        group.notes.push(note);
        normalize_legacy_project(&mut project);
        let group = &project.tracks[0].main_group;
        assert!(!group.parameters.contains_key("dF0Vbr"));
        assert!(!group.notes[0].attributes.contains_key("dF0Vbr"));
        assert_eq!(group.notes[0].attributes["dF0VbrMod"], json!(1.2));
    }
}
