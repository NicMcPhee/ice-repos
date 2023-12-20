use std::collections::BTreeMap;

use chrono::{DateTime, Local};

use serde::Deserialize;

use yewdux::prelude::*;

pub type RepoId = usize;

// TODO: Can we use `AttrValue` instead of `String` here
// and in other places where there are properties?
// I'm not sure what would be necessary here since
// serde is filling these in, but maybe for other
// properties. `AttrValue::from(string)` may be
// important in doing the necessary conversions.
// `AttrValue` is supposed to be more efficient
// because cloning `String`s can be expensive.
// https://yew.rs/docs/concepts/components/properties#memoryspeed-overhead-of-using-properties
#[derive(Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct Repository {
    pub id: RepoId,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub updated_at: DateTime<Local>,
    pub pushed_at: DateTime<Local>,
    // #[serde(flatten)]
    // extras: HashMap<String, Value>,
}

pub struct DesiredArchiveState {
    pub id: RepoId,
    pub desired_archive_state: bool,
}

// TODO: I think we may want to ultimately get rid of
//   this struct. It's not needed in the Paginator anymore,
//   and we may not need it in the `OrganizationEntry` component,
//   but I'm not 100% about that.
// TODO: Can we use `AttrValue` instead of `String` here?
// `AttrValue` is supposed to be more efficient
// because cloning `String`s can be expensive.
// https://yew.rs/docs/concepts/components/properties#memoryspeed-overhead-of-using-properties
#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct Organization {
    pub name: Option<String>,
}

// TODO: Add an `AlreadyArchived` option here and keep all the
//   the repositories in all the maps regardless of whether they
//   were archived in advance.
/// The desired state for a given repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesiredState {
    /// This repository was already archived and its state can't be change.
    AlreadyArchived,
    /// We have chosen in the pagination view to _not_ archive this repository.
    Keep,
    /// We have chosen in the pagination view to archive this repository.
    Archive,
    /// We have changed from "to archive" to "don't archive" in the review view.
    KeptInReview,
}

impl DesiredState {
    /// Convert a boolean, essentially the toggle state of a checkbox in the
    /// Paginator component and convert it into an `ArchiveState`. In the
    /// paginator, we want to use the `Skip` state to indicate that we do not
    /// want to see this archive in the review phase.
    #[must_use]
    pub const fn from_paginator_state(b: bool) -> Self {
        if b {
            Self::Archive
        } else {
            Self::Keep
        }
    }

    /// Convert a boolean, essentially the toggle state of a checkbox in the
    /// Review & Submit component and convert it into an `ArchiveState`. In
    /// the review, we want to use the `SkippedInReview` to indicate that we
    /// do want to continue to see this archive in the review phase.
    #[must_use]
    pub const fn from_review_state(b: bool) -> Self {
        if b {
            Self::Archive
        } else {
            Self::KeptInReview
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct DesiredStateMap {
    // Map from the repository ID as a key, to a pair
    // containing the Repository struct and a boolean
    // indicating whether we want to archive that repository
    // or not.
    pub map: BTreeMap<RepoId, (Repository, DesiredState)>,
}

impl DesiredStateMap {
    pub fn with_repos(&mut self, repositories: &[Repository]) -> &mut Self {
        for repo in repositories {
            let initial_state = if repo.archived {
                DesiredState::AlreadyArchived
            } else {
                DesiredState::Archive
            };
            self.map
                .entry(repo.id)
                .or_insert((repo.clone(), initial_state));
        }
        self
    }

    #[must_use]
    pub fn get_desired_state(&self, id: RepoId) -> Option<bool> {
        self.map
            .get(&id)
            .map(|(_, desired_state)| matches!(desired_state, DesiredState::Archive))
    }

    pub fn update_desired_state(&mut self, id: RepoId, desired_state: DesiredState) -> &mut Self {
        web_sys::console::log_1(&format!("Updating {id} to {desired_state:?}").into());
        self.map.entry(id).and_modify(|p| p.1 = desired_state);
        web_sys::console::log_1(&format!("The resulting map was {self:?}").into());
        self
    }

    /// # Panics
    ///
    /// Will panic `repo_id` isn't in the `ArchiveStateMap`.    
    #[must_use]
    pub fn get_repo(&self, repo_id: RepoId) -> &Repository {
        assert!(
            self.map.contains_key(&repo_id),
            "Repository key {repo_id} not found in StateMap"
        );
        #[allow(clippy::unwrap_used)]
        self.map.get(&repo_id).map(|p| &p.0).unwrap()
    }

    pub fn get_repos_to_review(&self) -> impl Iterator<Item = &Repository> {
        self.map.values().filter_map(|(repo, desired_state)| {
            (*desired_state != DesiredState::AlreadyArchived
                || *desired_state != DesiredState::Keep)
                .then_some(repo)
        })
    }

    #[must_use]
    pub fn get_repo_ids_to_review(&self) -> Vec<RepoId> {
        self.get_repos_to_review().map(|r| r.id).collect()
    }

    #[must_use]
    pub fn get_owned_repos_to_review(&self) -> Vec<Repository> {
        self.get_repos_to_review().cloned().collect()
    }

    pub fn get_repos_to_archive(&self) -> impl Iterator<Item = &Repository> {
        self.map
            .values()
            .filter_map(|(repo, to_archive)| (*to_archive == DesiredState::Archive).then_some(repo))
    }
}
