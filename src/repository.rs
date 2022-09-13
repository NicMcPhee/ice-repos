#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::collections::HashMap;

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
pub struct AppState {
    pub organization: Option<String>,
    pub archive_state_map: HashMap<usize, (Repository, bool)>
}
