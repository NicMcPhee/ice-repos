use std::collections::HashMap;

use yewdux::store::Store;

use crate::repository::RepoId;

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
