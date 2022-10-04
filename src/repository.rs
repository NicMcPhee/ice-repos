use std::collections::{BTreeMap, HashMap};

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
    pub desired_archive_state: bool
}

// TODO: Can we use `AttrValue` instead of `String` here?
// `AttrValue` is supposed to be more efficient
// because cloning `String`s can be expensive.
// https://yew.rs/docs/concepts/components/properties#memoryspeed-overhead-of-using-properties
#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct Organization {
    pub name: Option<String>
}

// TODO: Add an `AlreadyArchived` option here and keep all the
//   the repositories in all the maps regardless of whether they
//   were archived in advance.
/// The desired state for a given repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveState {
    /// We have chosen in the pagination view to _not_ archive this repository.
    Skip,
    /// We have chosen in the pagination view to archive this repository.
    Archive,
    /// We have changed from "to archive" to "don't archive" in the review view.
    SkippedInReview
}

impl ArchiveState {
    /// Convert a boolean, essentially the toggle state of a checkbox in the
    /// Paginator component and convert it into an `ArchiveState`. In the
    /// paginator, we want to use the `Skip` state to indicate that we do not
    /// want to see this archive in the review phase.
    #[must_use]
    pub const fn from_paginator_state(b: bool) -> Self {
        if b {
            Self::Archive
        } else {
            Self::Skip
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
            Self::SkippedInReview
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct ArchiveStateMap {
    // Map from the repository ID as a key, to a pair
    // containing the Repository struct and a boolean
    // indicating whether we want to archive that repository
    // or not.
    pub map: BTreeMap<RepoId, (Repository, ArchiveState)>
}

impl ArchiveStateMap {
    pub fn with_repos(&mut self, repositories: &[Repository]) -> &mut Self {
        for repo in repositories {
            if !repo.archived {
                self.map.entry(repo.id).or_insert((repo.clone(), ArchiveState::Archive));
            }
        }
        self
    }

    #[must_use]
    pub fn get_desired_state(&self, id: RepoId) -> Option<bool> {
        self.map
            .get(&id)
            .map(|(_, archive_state)| matches!(archive_state, ArchiveState::Archive))
    }

    pub fn update_desired_state(&mut self, id: RepoId, desired_archive_state: ArchiveState) -> &mut Self {
        web_sys::console::log_1(&format!("Updating {id} to {desired_archive_state:?}").into());
        self.map.entry(id).and_modify(|p| { p.1 = desired_archive_state });
        web_sys::console::log_1(&format!("The resulting map was {self:?}").into());
        self
    }

    /// # Panics
    ///
    /// Will panic `repo_id` isn't in the `ArchiveStateMap`.    
    #[must_use]
    pub fn get_repo(&self, repo_id: RepoId) -> &Repository {
        assert!(self.map.contains_key(&repo_id), "Repository key {repo_id} not found in ArchiveStateMap");
        #[allow(clippy::unwrap_used)]
        self.map.get(&repo_id).map(|p| &p.0).unwrap()
    }

    pub fn get_repos_to_review(&self) -> impl Iterator<Item = &Repository> {
        self.map
            .values()
            .filter_map(|(repo, to_archive)| {
                (*to_archive != ArchiveState::Skip).then_some(repo)
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
            .filter_map(|(repo, to_archive)| {
                (*to_archive == ArchiveState::Archive).then_some(repo)
            })
    }
}

pub type PageNumber = usize;

#[derive(Default, Store, Eq, PartialEq, Clone)]
pub struct PageRepoMap {
    map: HashMap<PageNumber, Vec<RepoId>>
}

impl PageRepoMap {
    #[must_use]
    pub fn has_seen_page(&self, page_number: PageNumber) -> bool {
        self.map.contains_key(&page_number)
    }

    #[must_use]
    pub fn get_repo_ids(&self, page_number: PageNumber) -> Option<Vec<RepoId>> {
        self.map.get(&page_number).cloned()
    }

    /// # Panics
    ///
    /// Will panic if `page_number` is already in the `PageRepoMap`. We
    /// shouldn't add the same page more than once, so if it's already
    /// there that indicates some kind of logical failure.
    pub fn add_page(&mut self, page_number: PageNumber, repo_ids: Vec<RepoId>) {
        assert!(!self.has_seen_page(page_number));
        self.map.insert(page_number, repo_ids);
    }
}