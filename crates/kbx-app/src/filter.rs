//! The filter in a few words, for the header's pill. The choices themselves live in
//! `actions.rs`'s filter page (D15).

use kbx_domain::search::FilterState;

use crate::catalog::Catalog;

/// The filter in a few words, for the pill: "All Entries", "Commands", "Commands, #k8s".
#[must_use]
pub fn filter_summary(filter: &FilterState, catalog: &Catalog) -> String {
    let parts: Vec<String> = filter
        .types
        .iter()
        .map(|t| {
            catalog
                .entry_type(t)
                .map_or_else(|| (*t).clone(), |kind| kind.label.clone())
        })
        .chain(filter.tags.iter().map(|t| format!("#{t}")))
        .collect();
    if parts.is_empty() {
        catalog.messages.all_entries.clone()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_owned()).collect()
    }

    #[test]
    fn summarizes_the_filter() {
        let catalog = Catalog::default();
        assert_eq!(filter_summary(&FilterState::default(), &catalog), "All Entries");
        assert_eq!(
            filter_summary(
                &FilterState {
                    types: strings(&["commands"]),
                    tags: strings(&["git"])
                },
                &catalog
            ),
            "Commands, #git"
        );
    }
}
