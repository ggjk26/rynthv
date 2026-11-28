use std::{collections::BTreeMap, path::Path, sync::OnceLock};

use sha2::{Digest, Sha256};

use super::{LoadedModel, TensorSummary, VoiceFile, VoiceInventory, inspect};
use crate::synthesis::voice::{Result, VoiceError, embedded, hex, nofs};

pub(super) fn load(path: &Path) -> Result<VoiceInventory> {
    let mut archive = nofs::NofsArchive::open(path)?;
    let description = nofs::describe(&mut archive)?;
    let mut digest = Sha256::new();
    digest.update(b"RynthV SV1 public metadata and model resources\0");
    digest.update(
        serde_json::to_vec(&description.configuration)
            .map_err(|error| VoiceError::new(error.to_string()))?,
    );
    let mut graphs = Vec::new();
    let mut trees = Vec::new();
    let mut tensors = BTreeMap::<_, TensorSummary>::new();
    for (index, role) in description.models {
        let bytes = archive.read(index, nofs::MAX_MODEL_BYTES)?;
        digest.update(role.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(&bytes);
        let records = embedded::parse(&bytes)?;
        graphs.push(inspect::inline_graph(&records, index, role, 0, &mut tensors)?);
        trees.push(records.into());
    }
    let content_hash = hex(&digest.finalize());
    let graph_count = graphs.len();
    let tensor_count = tensors.values().map(|summary| summary.resources).sum();
    let resource_count = archive.records.len();
    let file = VoiceFile {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        bytes: usize::try_from(archive.length)
            .map_err(|_| VoiceError::new("SV1 file size exceeds address space"))?,
        sha256: content_hash.clone(),
        sha256_scope: "public_metadata_and_models",
        resources: resource_count,
        borrowed_resources: resource_count,
        encoded_resources: 0,
        // NOFS has structural checks, not DNNI v3 signatures or resource digests.
        header_signature_equal_branch_verified: false,
        index_digest_verified: false,
        all_resource_digests_verified: false,
        graphs,
        referenced_modules: graph_count,
        unreferenced_resources: resource_count - graph_count,
        tensors,
        configuration_parsed: true,
    };
    Ok(VoiceInventory {
        database_id: nofs::database_id(path)?,
        directory: archive.path,
        display_name: description.configuration.name.clone(),
        content_hash,
        resource_count,
        graph_count,
        tensor_count,
        files: vec![file],
        configuration: Some(description.configuration),
        _models: vec![LoadedModel::Inline { trees }],
        flat_profile: None,
        preparation: OnceLock::new(),
    })
}
