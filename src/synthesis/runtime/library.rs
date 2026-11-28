use super::{Runtime, events::WakeOnExit};
use crate::synthesis::{
    settings::{SynthesisSettings, VoiceDirectory, DurationModelSlot, VoiceSelection},
    voice::{VoiceAvailability, VoiceCatalog, VoiceCatalogEntry, flat::{FlatVoice, probe::FlatPreflight}},
};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
};

pub struct VoiceLibrary {
    pub settings: SynthesisSettings,
    catalog: Option<Arc<VoiceCatalog>>,
    scan: Option<Receiver<LibraryScan>>,
    flat_probe: Option<FlatProbe>,
    flat_probe_result: Option<(PathBuf, Result<FlatPreflight, String>)>,
    catalog_error: Option<String>,
    settings_error: Option<String>,
    invalid_settings: bool,
}

struct FlatProbe {
    path: PathBuf,
    receiver: Receiver<Result<FlatPreflight, String>>,
    invalidated: bool,
}

struct LibraryScan {
    catalog: VoiceCatalog,
    error: Option<String>,
}

fn scan_roots(settings: &SynthesisSettings) -> LibraryScan {
    let mut catalog = VoiceCatalog::default();
    let mut errors = Vec::new();
    for (label, root) in settings.database_roots() {
        match VoiceCatalog::scan_with_resources(
            root,
            settings.flat_models.as_deref(),
            settings.sv2_duration_model.as_deref(),
            settings.flat_timing_model.as_deref(),
        ) {
            Ok(found) => {
                catalog.entries.extend(found.entries);
                catalog.warnings.extend(
                    found
                        .warnings
                        .into_iter()
                        .map(|warning| format!("{label} {}：{warning}", root.display())),
                );
            }
            Err(error) => errors.push(format!(
                "无法扫描 {label} 声库目录 {}：{error}",
                root.display()
            )),
        }
    }
    catalog.entries.sort_by(|left, right| {
        left.database_id
            .cmp(&right.database_id)
            .then_with(|| left.revision.cmp(&right.revision))
            .then_with(|| left.directory.cmp(&right.directory))
    });
    catalog.entries.dedup_by(|left, right| {
        left.database_id == right.database_id
            && left.revision == right.revision
            && left.directory == right.directory
    });
    for duplicates in catalog.entries.chunk_by_mut(|left, right| {
        left.database_id == right.database_id && left.revision == right.revision
    }) {
        if duplicates.len() > 1 {
            let message = format!(
                "声库 {}/{} 对应多个文件，请检查声库目录配置。",
                duplicates[0].database_id, duplicates[0].revision
            );
            errors.push(message.clone());
            for entry in duplicates {
                entry.availability = VoiceAvailability::Unsupported(message.clone());
            }
        }
    }
    catalog.entries.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.database_id.cmp(&right.database_id))
            .then_with(|| right.revision.cmp(&left.revision))
    });
    LibraryScan {
        catalog,
        error: (!errors.is_empty()).then(|| errors.join("\n")),
    }
}

impl VoiceLibrary {
    pub(super) fn new(repaint: eframe::egui::Context) -> Self {
        let (settings, settings_error) = match SynthesisSettings::load() {
            Ok(settings) => (settings, None),
            Err(error) => (SynthesisSettings::default(), Some(error)),
        };
        let invalid_settings = settings_error.is_some();
        let mut library = Self {
            settings,
            catalog: None,
            scan: None,
            flat_probe: None,
            flat_probe_result: None,
            catalog_error: None,
            settings_error,
            invalid_settings,
        };
        if !invalid_settings {
            library.rescan(repaint);
        }
        library
    }

    pub fn catalog(&self) -> Option<&VoiceCatalog> {
        self.catalog.as_deref()
    }

    pub fn catalog_snapshot(&self) -> Option<Arc<VoiceCatalog>> {
        self.catalog.clone()
    }

    pub fn is_scanning(&self) -> bool {
        self.scan.is_some()
    }

    pub fn flat_probe_running(&self) -> bool {
        self.flat_probe.is_some()
    }

    pub fn flat_probe_path(&self) -> Option<&Path> {
        self.flat_probe
            .as_ref()
            .filter(|probe| !probe.invalidated)
            .map(|probe| probe.path.as_path())
    }

    pub fn flat_probe_for(&self, path: &Path) -> Option<&Result<FlatPreflight, String>> {
        self.flat_probe_result.as_ref()
            .filter(|(checked, _)| checked == path)
            .map(|(_, report)| report)
    }

    fn start_flat_probe(
        &mut self,
        path: PathBuf,
        repaint: eframe::egui::Context,
    ) -> Result<(), String> {
        if self.flat_probe.is_some() {
            return Err("已有 Flat 模型预检正在执行".into());
        }
        if !path.is_file() || !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("nofs")) {
            return Err("请选择一个 Flat JSON NOFS，而不是普通 SV1 NOFS 或不可解析的二进制包".into());
        }
        let models = self.settings.flat_models.clone();
        let timing = self.settings.flat_timing_model.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let source = path.clone();
        thread::Builder::new()
            .name("flat-model-preflight".into())
            .spawn(move || {
                let _wake = WakeOnExit(repaint);
                let report = FlatVoice::open(&source, models.as_deref())
                    .map(|voice| voice.preflight(timing.as_deref()))
                    .map_err(|error| error.to_string());
                let _ = tx.send(report);
            })
            .map_err(|error| format!("无法启动 Flat 模型预检：{error}"))?;
        self.flat_probe_result = None;
        self.flat_probe = Some(FlatProbe {
            path,
            receiver: rx,
            invalidated: false,
        });
        Ok(())
    }

    pub fn catalog_error(&self) -> Option<&str> {
        self.catalog_error.as_deref()
    }

    pub fn settings_error(&self) -> Option<&str> {
        self.settings_error.as_deref()
    }

    pub fn invalid_settings(&self) -> bool {
        self.invalid_settings
    }

    fn recover_invalid_settings(&mut self) -> Result<PathBuf, String> {
        if !self.invalid_settings {
            return Err("当前合成设置无需恢复。".into());
        }
        let backup = SynthesisSettings::backup_and_reset_invalid()?;
        self.settings = SynthesisSettings::default();
        self.invalid_settings = false;
        self.settings_error = None;
        self.catalog_error = None;
        Ok(backup)
    }

    fn update(&mut self, settings: SynthesisSettings) -> bool {
        if self.invalid_settings {
            return false;
        }
        match settings.save() {
            Ok(()) => {
                self.settings = settings;
                self.settings_error = None;
                true
            }
            Err(error) => {
                self.settings_error = Some(error);
                false
            }
        }
    }

    pub fn entry(&self, selection: &VoiceSelection) -> Option<&VoiceCatalogEntry> {
        self.catalog.as_ref()?.entry(selection)
    }
    fn rescan(&mut self, repaint: eframe::egui::Context) {
        // Dropping the receiver prevents an earlier directory scan from
        // replacing the catalog after the user changes paths again.
        self.scan = None;
        if let Some(probe) = &mut self.flat_probe {
            probe.invalidated = true;
        }
        self.flat_probe_result = None;
        self.catalog = None;
        self.catalog_error = None;
        if self.invalid_settings {
            return;
        }
        if !self.settings.has_databases() {
            return;
        }
        let settings = self.settings.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        match thread::Builder::new()
            .name("voice-library-scan".into())
            .spawn(move || {
                let _wake = WakeOnExit(repaint);
                let sender = tx;
                let result = scan_roots(&settings);
                let _ = sender.send(result);
            }) {
            Ok(_) => self.scan = Some(rx),
            Err(error) => self.catalog_error = Some(format!("无法启动声库扫描：{error}")),
        }
    }

    fn poll_flat_probe(&mut self) {
        let Some(probe) = &self.flat_probe else {
            return;
        };
        let result = match probe.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Disconnected) => Err("Flat 模型预检线程意外退出".into()),
            Err(TryRecvError::Empty) => return,
        };
        if let Some(probe) = self.flat_probe.take()
            && !probe.invalidated
        {
            self.flat_probe_result = Some((probe.path, result));
        }
    }

    fn poll(&mut self) -> bool {
        self.poll_flat_probe();
        let mut changed = false;
        if let Some(scan) = &self.scan {
            match scan.try_recv() {
                Ok(result) => {
                    self.scan = None;
                    self.catalog = Some(Arc::new(result.catalog));
                    self.catalog_error = result.error;
                    changed = true;
                }
                Err(TryRecvError::Disconnected) => {
                    self.scan = None;
                    self.catalog_error = Some("声库扫描线程意外终止。".into());
                    changed = true;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        changed
    }
}

impl Runtime {
    /// Validate a Flat voice's actual installed model archives off the UI thread.
    /// This does not mark Flat voice synthesis as supported.
    pub fn inspect_flat_models(&mut self, path: &Path) -> Result<(), String> {
        self.library.start_flat_probe(path.to_owned(), self.repaint.clone())
    }

    /// Reset malformed resource settings only after explicit UI confirmation.
    /// The original configuration remains available in a timestamp-independent
    /// unique backup for manual path recovery.
    pub fn recover_invalid_voice_settings(&mut self) -> Result<PathBuf, String> {
        let backup = self.library.recover_invalid_settings()?;
        self.clear_voices();
        self.library.rescan(self.repaint.clone());
        Ok(backup)
    }

    /// Explicitly apply the detected native Windows voicebank roots.
    /// Do not retain stale legacy roots which would scan old folders too.
    pub fn use_installed_voice_roots(&mut self) -> Result<usize, String> {
        let found = SynthesisSettings::installed_windows_roots()?;
        let count = [found.databases.as_ref(), found.sv1_databases.as_ref(), found.flat_databases.as_ref()]
            .into_iter().flatten().count();
        let mut settings = self.library.settings.clone();
        // Preserve a manually configured directory when the corresponding
        // standard location does not exist (e.g. Documents redirected to OneDrive).
        if found.databases.is_some() {
            settings.databases = found.databases;
        }
        if found.sv1_databases.is_some() {
            settings.sv1_databases = found.sv1_databases;
        }
        if found.flat_databases.is_some() {
            settings.flat_databases = found.flat_databases;
        }
        if found.flat_models.is_some() {
            settings.flat_models = found.flat_models;
        }
        settings.roots = None;
        settings.selected_voice = None;
        if !self.library.update(settings) {
            return Err(self.library.settings_error().unwrap_or("无法保存声库根目录").to_owned());
        }
        self.rescan_voices();
        Ok(count)
    }

    /// Assign or clear an individual Duration DNNI. The Flat model is used
    /// for metadata/dependency lookup; only a validated SV2 Duration resource
    /// can be injected into the native SV2 synthesis pipeline.
    pub fn set_duration_file(
        &mut self,
        slot: DurationModelSlot,
        path: Option<PathBuf>,
    ) -> Result<(), String> {
        if self.library.invalid_settings() {
            return Err("合成配置无法读取，请先备份并恢复 synthesis.json。".into());
        }
        if let Some(path) = &path {
            if !path.is_file() {
                return Err(format!("时长模型文件不存在或不可读取：{}", path.display()));
            }
            if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dnni")) {
                return Err(format!("请选择 .dnni 文件，而不是文件夹：{}", path.display()));
            }
            // Do a lightweight header check synchronously; full DNNI
            // resource-authenticity checks run off the UI thread on rescan.
            if slot == DurationModelSlot::Sv2 {
                let mut header = [0u8; 8];
                std::fs::File::open(path)
                    .and_then(|mut file| file.read_exact(&mut header))
                    .map_err(|error| format!("无法读取 SV2 Duration DNNI 文件头：{error}"))?;
                if header != [0xff, 0, 0xca, 0x7f, 3, 0, 0, 0] {
                    return Err("SV2 Duration 需要 DNNI v3 文件；Flat Timing 与 SV2 Duration 不能直接混用。".into());
                }
            }
        }
        let mut settings = self.library.settings.clone();
        let target = match slot {
            DurationModelSlot::Sv2 => &mut settings.sv2_duration_model,
            DurationModelSlot::Flat => &mut settings.flat_timing_model,
        };
        if *target == path {
            return Ok(());
        }
        *target = path;
        if !self.library.update(settings) {
            return Err(self.library.settings_error().unwrap_or(
                "无法保存时长模型设置",
            ).to_owned());
        }
        self.rescan_voices();
        Ok(())
    }

    pub fn dictionaries(&self) -> Option<&Path> {
        self.library.settings.dictionaries.as_deref()
    }
    /// Apply one root from either the native folder picker or a pasted path.
    /// Validate before touching the saved configuration and report failures to
    /// the UI rather than silently keeping the previous root.
    pub fn set_voice_directory(
        &mut self,
        directory: VoiceDirectory,
        path: Option<PathBuf>,
    ) -> Result<(), String> {
        if self.library.invalid_settings() {
            return Err("声库设置文件无法读取；请先备份并重置损坏的 synthesis.json。".into());
        }
        if let Some(path) = &path {
            let metadata = std::fs::metadata(path).map_err(|error| {
                format!("无法访问声库目录 {}：{error}", path.display())
            })?;
            if !metadata.is_dir() {
                return Err(format!("选择的路径不是文件夹：{}", path.display()));
            }
        }
        let mut settings = self.library.settings.clone();
        let target = match directory {
            VoiceDirectory::Sv1 => &mut settings.sv1_databases,
            VoiceDirectory::Sv2 => &mut settings.databases,
            VoiceDirectory::Flat => &mut settings.flat_databases,
            VoiceDirectory::FlatModels => &mut settings.flat_models,
        };
        if *target == path {
            return Ok(());
        }
        *target = path;
        // Explicit modern paths supersede old, untyped roots. Otherwise a
        // stale legacy directory can continue generating scan errors after a
        // user has successfully selected a new SV1/SV2 location.
        settings.roots = None;
        if !self.library.update(settings) {
            return Err(self.library.settings_error().unwrap_or(
                "声库目录保存失败，请检查 synthesis.json 的权限。"
            ).to_owned());
        }
        self.rescan_voices();
        Ok(())
    }
    pub fn set_databases(&mut self, path: PathBuf) -> Result<(), String> {
        self.set_voice_directory(VoiceDirectory::Sv2, Some(path))
    }
    pub fn set_sv1_databases(&mut self, path: PathBuf) -> Result<(), String> {
        self.set_voice_directory(VoiceDirectory::Sv1, Some(path))
    }
    pub fn set_flat_databases(&mut self, path: PathBuf) -> Result<(), String> {
        self.set_voice_directory(VoiceDirectory::Flat, Some(path))
    }
    pub fn set_flat_models(&mut self, path: PathBuf) -> Result<(), String> {
        self.set_voice_directory(VoiceDirectory::FlatModels, Some(path))
    }
    pub fn set_dictionaries(&mut self, path: PathBuf) {
        let mut settings = self.library.settings.clone();
        settings.dictionaries = Some(path);
        if self.library.update(settings) {
            self.clear_voices();
        }
    }
    pub fn use_builtin_dictionaries(&mut self) {
        let mut settings = self.library.settings.clone();
        settings.dictionaries = None;
        if self.library.update(settings) {
            self.clear_voices();
        }
    }
    pub fn rescan_voices(&mut self) {
        self.clear_voices();
        self.library.rescan(self.repaint.clone());
    }
    pub(super) fn poll_library(&mut self) {
        if self.library.poll() {
            self.last_revision = None;
        }
    }
}
