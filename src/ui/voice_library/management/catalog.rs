use super::{
    super::{family_name, language_name},
    Sort, State,
};
use crate::{
    synthesis::{
        settings::VoiceSelection,
        voice::{VoiceCatalog, VoiceCatalogEntry},
    },
    ui::settings::UiLanguage,
};
use std::collections::BTreeMap;

pub(super) fn selection(entry: &VoiceCatalogEntry) -> VoiceSelection {
    VoiceSelection {
        database_id: entry.database_id.clone(),
        revision: entry.revision,
    }
}

pub(super) fn is_selected(entry: &VoiceCatalogEntry, current: Option<&VoiceSelection>) -> bool {
    current
        .is_some_and(|key| key.database_id == entry.database_id && key.revision == entry.revision)
}

pub(super) fn entries<'a>(
    catalog: &'a VoiceCatalog,
    state: &State,
    current: Option<&VoiceSelection>,
    language: UiLanguage,
) -> Vec<&'a VoiceCatalogEntry> {
    let mut banks: BTreeMap<&str, &VoiceCatalogEntry> = BTreeMap::new();
    let query = state.query.trim().to_lowercase();
    let rank = |entry: &VoiceCatalogEntry| {
        (
            state.versions.get(&entry.database_id) == Some(&entry.revision),
            is_selected(entry, current),
            entry.availability.is_loadable(),
            entry.revision,
        )
    };
    for entry in &catalog.entries {
        if !state.language.is_empty() && !entry.languages.contains(&state.language)
            || !state.vendor.is_empty() && entry.vendor != state.vendor
        {
            continue;
        }
        let matches = query.is_empty()
            || entry.display_name.to_lowercase().contains(&query)
            || entry.vendor.to_lowercase().contains(&query)
            || family_name(entry, language).to_lowercase().contains(&query)
            || entry.languages.iter().any(|s| {
                language_name(s, language).to_lowercase().contains(&query)
                    || s.to_lowercase().contains(&query)
            });
        if !matches {
            continue;
        }
        let best = banks.entry(&entry.database_id).or_insert(entry);
        if rank(entry) > rank(best) {
            *best = entry;
        }
    }
    let mut entries: Vec<_> = banks.into_values().collect();
    entries.sort_by(|a, b| {
        let names = || {
            a.display_name
                .cmp(&b.display_name)
                .then_with(|| a.database_id.cmp(&b.database_id))
        };
        match state.sort {
            Sort::Default => b
                .availability
                .is_loadable()
                .cmp(&a.availability.is_loadable())
                .then_with(names),
            Sort::Name => names(),
            Sort::NameReverse => names().reverse(),
            Sort::Version => b.revision.cmp(&a.revision).then_with(names),
        }
    });
    entries
}
