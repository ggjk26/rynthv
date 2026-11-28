use super::Editor;
use crate::model::Language;

impl Editor {
    /// Override one group instance without touching its sibling groups or the
    /// track's default voice. None removes the override for linked instances.
    pub fn set_reference_voice(
        &mut self,
        track_id: crate::model::EntityId,
        reference_id: crate::model::EntityId,
        singer: Option<(&str, &str, u32)>,
    ) -> Result<(), String> {
        let Some(track) = self.project.tracks.iter().find(|track| track.id == track_id) else {
            return Err("目标音轨不存在".into());
        };
        if track.is_instrumental() {
            return Err("伴奏轨不能设置歌手".into());
        }
        if !track.references().any(|reference| {
            reference.id == reference_id && reference.database.is_object()
        }) {
            return Err("音符组不存在或 database 字段无效".into());
        }
        self.begin("设置音符组声库");
        let track = self.project.tracks.iter_mut()
            .find(|track| track.id == track_id).expect("validated track");
        let reference = std::iter::once(&mut track.main_ref)
            .chain(track.groups.iter_mut())
            .find(|reference| reference.id == reference_id)
            .expect("validated reference");
        let database = reference.database.as_object_mut().expect("validated database");
        database.remove("id");
        match singer {
            Some((id, name, revision)) => {
                database.insert("id".into(), id.into());
                database.insert("name".into(), name.into());
                database.insert("version".into(), revision.to_string().into());
            }
            None => {
                database.insert("name".into(), "".into());
                database.insert("version".into(), "-2".into());
            }
        }
        for key in ["language", "phoneset", "backendType"] {
            database.insert(key.into(), "".into());
        }
        self.finish();
        Ok(())
    }

    /// Apply one voice to the selected group instances as one undoable edit.
    /// The targets are validated before any group data is changed.
    pub fn set_references_voice(
        &mut self,
        ids: &std::collections::BTreeSet<crate::model::EntityId>,
        singer: Option<(&str, &str, u32)>,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let targets = self.project.tracks.iter()
            .filter(|track| !track.is_instrumental())
            .flat_map(|track| track.references())
            .filter(|reference| ids.contains(&reference.id))
            .collect::<Vec<_>>();
        if targets.len() != ids.len() || targets.iter().any(|r| !r.database.is_object()) {
            return Err("所选音符组包含无效歌声目标".into());
        }
        self.begin("设置音符组声库");
        for track in self.project.tracks.iter_mut()
            .filter(|track| !track.is_instrumental()) {
            for reference in std::iter::once(&mut track.main_ref)
                .chain(track.groups.iter_mut())
                .filter(|reference| ids.contains(&reference.id)) {
                let database = reference.database.as_object_mut().expect("validated database");
                database.remove("id");
                match singer {
                    Some((id, name, revision)) => {
                        database.insert("id".into(), id.into());
                        database.insert("name".into(), name.into());
                        database.insert("version".into(), revision.to_string().into());
                    }
                    None => {
                        database.insert("name".into(), "".into());
                        database.insert("version".into(), "-2".into());
                    }
                }
                for key in ["language", "phoneset", "backendType"] {
                    database.insert(key.into(), "".into());
                }
            }
        }
        self.finish();
        Ok(())
    }

    /// Apply one singer to several vocal tracks in a single undo transaction.
    /// Existing per-reference values follow the standard track voice operation.
    pub fn set_tracks_voice(
        &mut self,
        ids: &std::collections::BTreeSet<crate::model::EntityId>,
        singer: Option<(&str, &str, u32)>,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let affected = self.project.tracks.iter()
            .filter(|track| ids.contains(&track.id))
            .collect::<Vec<_>>();
        if affected.len() != ids.len() || affected.iter().any(|track| track.is_instrumental()) {
            return Err("所选轨道包含无效或伴奏轨道".into());
        }
        if affected.iter().flat_map(|track| track.references())
            .any(|reference| !reference.database.is_object())
        {
            return Err("所选音轨的歌声 database 字段必须是对象".into());
        }
        self.begin("批量切换音轨歌手");
        for track in self.project.tracks.iter_mut().filter(|track| ids.contains(&track.id)) {
            for reference in std::iter::once(&mut track.main_ref).chain(&mut track.groups) {
                let database = reference.database.as_object_mut().expect("validated database");
                database.remove("id");
                let (name, revision) = match singer {
                    Some((id, name, revision)) => {
                        database.insert("id".into(), id.into());
                        (name, revision.to_string())
                    }
                    None => ("", "-2".to_string()),
                };
                database.insert("name".into(), name.into());
                database.insert("version".into(), revision.into());
                for key in ["language", "phoneset", "backendType"] {
                    database.insert(key.into(), "".into());
                }
            }
        }
        self.finish();
        Ok(())
    }

    pub fn set_track_voice(&mut self, singer: Option<(&str, &str, u32)>) -> Result<(), String> {
        if self.current_track().is_instrumental() {
            return Err("伴奏轨不能设置歌手".into());
        }
        if self
            .current_track()
            .references()
            .any(|r| !r.database.is_object())
        {
            return Err("音轨的歌声 database 字段必须是对象".into());
        }
        self.begin("切换音轨歌手");
        let track = self
            .project
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track)
            .expect("selected track exists");
        for reference in std::iter::once(&mut track.main_ref).chain(&mut track.groups) {
            let database = reference
                .database
                .as_object_mut()
                .expect("validated database");
            database.remove("id");
            let (name, version) = match singer {
                Some((id, name, version)) => {
                    database.insert("id".into(), id.into());
                    (name, version.to_string())
                }
                None => ("", "-2".to_owned()),
            };
            database.insert("name".into(), name.into());
            database.insert("version".into(), version.into());
            // These are bank defaults; explicit user language overrides stay intact.
            for key in ["language", "phoneset", "backendType"] {
                database.insert(key.into(), "".into());
            }
        }
        self.finish();
        Ok(())
    }

    /// With a group selected, edit only that instance. Otherwise edit the track
    /// defaults, which are inherited by subsequently created groups.
    pub fn set_voice_language(&mut self, language: Option<Language>) -> Result<(), String> {
        let target = self.active_ref().unwrap_or(&self.current_track().main_ref);
        let target_id = target.id;
        let database = target
            .database
            .as_object()
            .ok_or("歌声的 database 字段必须是对象")?;
        let value = language.map_or("", Language::id);
        let alphabet = language.map_or("", Language::phoneme_alphabet);
        if database
            .get("languageOverride")
            .and_then(serde_json::Value::as_str)
            == Some(value)
            && database
                .get("phonesetOverride")
                .and_then(serde_json::Value::as_str)
                == Some(alphabet)
        {
            return Ok(());
        }
        self.begin("切换歌声语言");
        let track = self
            .project
            .tracks
            .iter_mut()
            .find(|track| track.id == self.track)
            .expect("selected track exists");
        let reference = std::iter::once(&mut track.main_ref)
            .chain(&mut track.groups)
            .find(|reference| reference.id == target_id)
            .expect("selected reference exists");
        let database = reference
            .database
            .as_object_mut()
            .expect("validated database");
        database.insert("languageOverride".into(), value.into());
        database.insert("phonesetOverride".into(), alphabet.into());
        self.finish();
        Ok(())
    }
}
