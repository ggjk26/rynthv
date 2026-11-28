use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, OnceLock},
};

use sha2::{Digest, Sha256};

use super::{LoadedModel, TensorSummary, VoiceFile, VoiceInventory, VoiceResources, inspect};
use crate::synthesis::voice::{
    Result, VoiceError,
    config::VoiceConfiguration,
    embedded,
    flat::{FlatVoice, inline, render},
    hex,
};

pub(super) fn load(path: &Path, resources: &VoiceResources) -> Result<VoiceInventory> {
    let voice = FlatVoice::open(path, resources.flat_models.as_deref())?;
    let profile = voice.render_profile()?;
    let configuration = VoiceConfiguration::from_flat(
        &voice,
        profile
            .variants
            .iter()
            .map(|(name, _)| name.clone())
            .collect(),
    );
    let mut digest = Sha256::new();
    digest.update(b"RynthV Flat NOFS-JSON and inline DDS stages\0");
    digest.update(
        serde_json::to_vec(&(&configuration, &profile))
            .map_err(|error| VoiceError::new(error.to_string()))?,
    );

    let mut files = Vec::new();
    let mut models = Vec::new();
    let mut tensor_count = 0;
    for stage in voice.render_models(resources.flat_timing.as_deref())? {
        let model = inline::read(stage.path, stage.root).map_err(|error| {
            VoiceError::new(format!(
                "Flat {} ({})：{error}",
                stage.role,
                stage.path.display()
            ))
        })?;
        let data = model.graph();
        let bytes = model.file_bytes();
        let hash = Sha256::digest(data);
        digest.update(stage.role.as_bytes());
        digest.update(hash);
        let encoded = usize::from(data.len() != bytes);
        let records = embedded::parse(data)?;
        let mut tensors = BTreeMap::<_, TensorSummary>::new();
        let graph = inspect::inline_graph(&records, 0, stage.role, encoded as u32, &mut tensors)?;
        tensor_count += tensors
            .values()
            .map(|summary| summary.resources)
            .sum::<usize>();
        files.push(VoiceFile {
            name: stage
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            bytes,
            sha256: hex(&hash),
            sha256_scope: "inline_dds_graph",
            resources: 1,
            borrowed_resources: 1 - encoded,
            encoded_resources: encoded,
            header_signature_equal_branch_verified: false,
            index_digest_verified: false,
            all_resource_digests_verified: false,
            graphs: vec![graph],
            referenced_modules: 1,
            unreferenced_resources: 0,
            tensors,
            configuration_parsed: false,
        });
        models.push(LoadedModel::Inline {
            trees: vec![records.into()],
        });
    }
    Ok(VoiceInventory {
        database_id: render::database_id(path)?,
        directory: path.to_owned(),
        display_name: voice.name,
        content_hash: hex(&digest.finalize()),
        resource_count: files.len(),
        graph_count: files.len(),
        tensor_count,
        files,
        configuration: Some(configuration),
        _models: models,
        flat_profile: Some(Arc::new(profile)),
        preparation: OnceLock::new(),
    })
}
