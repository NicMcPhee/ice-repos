use std::{collections::BTreeMap, rc::Rc};

use chrono::{DateTime, Local};

use serde::Deserialize;

use yewdux::prelude::*;

pub type RepoId = usize;

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct Organization {
    name: Option<Rc<str>>,
    pub repositories: Repositories,
}

impl Organization {
    /// Update the organization name. This will also reset the repositories, because we are
    /// changing organizations. A separate call to `load_organization` must be made to actually
    /// load the new organization's repositories.
    pub fn set_name(&mut self, name: Rc<str>) {
        self.name = Some(name);
        self.repositories = Default::default();
    }

    pub const fn name(&self) -> Option<&Rc<str>> {
        self.name.as_ref()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Repository {
    pub info: RepositoryInfo,
    pub archive_state: ArchiveState,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct RepositoryInfo {
    pub id: RepoId,
    pub name: Rc<str>,
    pub description: Option<Rc<str>>,
    pub archived: bool,
    pub updated_at: DateTime<Local>,
    pub pushed_at: DateTime<Local>,
    // #[serde(flatten)]
    // extras: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Repositories(BTreeMap<RepoId, Repository>);

impl Repositories {
    /// Updates the repositories with the given `repos`. This will overwrite any existing
    /// repositories with the same ID.
    pub fn update(&mut self, repos: impl IntoIterator<Item = Repository>) {
        for repo in repos {
            self.0.insert(repo.info.id, repo);
        }
    }

    /// Selects a range of repositories, filtered by the given `filter`. The range is selected
    /// _after_ filtering.
    ///
    /// # Panics
    /// Panics if `range.start > range.end`.
    pub fn select<'a>(
        &'a self,
        range: std::ops::Range<usize>,
        filter: &'a RepoFilter,
    ) -> impl Iterator<Item = &Repository> + 'a {
        assert!(range.start <= range.end, "range start must be <= range end");
        self.0
            .values()
            .filter(|repo| filter.call(repo))
            .skip(range.start)
            .take(range.end - range.start)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, id: &RepoId) -> Option<&Repository> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: &RepoId) -> Option<&mut Repository> {
        self.0.get_mut(id)
    }
}

/// The desired state for a given repository.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ArchiveState {
    /// This repository was already archived and its state can't be change.
    AlreadyArchived,
    /// We have chosen in the pagination view to _not_ archive this repository.
    Keep,
    /// We have chosen in the pagination view to archive this repository.
    Archive,
    /// We have changed from "to archive" to "don't archive" in the review view.
    KeptInReview,
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

/// Filters repositories based on some criteria.
///
/// # Example
/// ```
/// use ice_repos::organization::RepoFilter;
///
/// let filter = RepoFilter::review_and_submit()
///     .and(|repo| repo.info.description.is_some())
///     .and(|repo| repo.info.name.starts_with("a"))
///     // Other filters can be unpacked to combine with the current filter.
///     .and(RepoFilter::all().unpack());
/// ```
#[derive(Clone)]
pub struct RepoFilter(Rc<dyn Fn(&Repository) -> bool>);
impl RepoFilter {
    pub fn new(filter: impl Fn(&Repository) -> bool + 'static) -> Self {
        Self(Rc::new(filter))
    }

    /// Filter for repositories to display in the review and submit page.
    pub fn review_and_submit() -> Self {
        Self::new(|repo| {
            [ArchiveState::Archive, ArchiveState::KeptInReview].contains(&repo.archive_state)
        })
    }

    /// Let all repositories through.
    pub fn all() -> Self {
        Self::new(|_| true)
    }

    pub fn and<F>(self, other: F) -> Self
    where
        F: Fn(&Repository) -> bool + 'static,
    {
        Self::new(move |repo| self.call(repo) && other(repo))
    }

    pub fn or<F>(self, other: F) -> Self
    where
        F: Fn(&Repository) -> bool + 'static,
    {
        Self::new(move |repo| self.call(repo) || other(repo))
    }

    pub fn unpack(self) -> impl Fn(&Repository) -> bool {
        move |repo| self.call(repo)
    }

    pub fn call(&self, repo: &Repository) -> bool {
        (self.0)(repo)
    }
}

impl PartialEq for RepoFilter {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<F> From<F> for RepoFilter
where
    F: Fn(&Repository) -> bool + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}
