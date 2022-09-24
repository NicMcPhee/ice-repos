use std::collections::BTreeMap;

use chrono::{DateTime, Local};

use serde::Deserialize;

use yewdux::prelude::*;

#[derive(Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct Repository {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub updated_at: DateTime<Local>,
    pub pushed_at: DateTime<Local>,

    // #[serde(flatten)]
    // extras: HashMap<String, Value>,
}

pub struct DesiredArchiveState {
    pub id: usize,
    pub desired_archive_state: bool
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct Organization {
    pub name: Option<String>
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct ArchiveStateMap {
    // Map from the repository ID as a key, to a pair
    // containing the Repository struct and a boolean
    // indicating whether we want to archive that repository
    // or not.
    pub map: BTreeMap<usize, (Repository, bool)>
}

impl ArchiveStateMap {
    pub fn with_repos(&mut self, repositories: &[Repository]) -> &mut Self {
        for repo in repositories {
            if !repo.archived {
                self.map.entry(repo.id).or_insert((repo.clone(), true));
            }
        }
        self
    }

    #[must_use]
    pub fn get_desired_state(&self, id: usize) -> Option<bool> {
        self.map
            .get(&id)
            .map(|p| p.1)
    }

    pub fn update_desired_state(&mut self, id: usize, desired_archive_state: bool) -> &mut Self {
        web_sys::console::log_1(&format!("Updating {id} to {desired_archive_state}").into());
        self.map.entry(id).and_modify(|p| { p.1 = desired_archive_state });
        web_sys::console::log_1(&format!("The resulting map was {self:?}").into());
        self
    }

    pub fn get_repos_to_archive(&self) -> impl Iterator<Item = &Repository> {
        self.map.values().filter_map(|(repo, to_archive)| to_archive.then_some(repo))
    }
}
