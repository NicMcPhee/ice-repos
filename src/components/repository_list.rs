#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;

use crate::repository::{Repository, DesiredArchiveState};
use crate::components::repository_paginator::ArchiveStateMap;
use crate::components::repository_card::RepositoryCard;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub repositories: Vec<Repository>,
    pub archive_state_map: UseStateHandle<ArchiveStateMap>,
    pub on_checkbox_change: Callback<DesiredArchiveState>
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let Props { repositories, archive_state_map, on_checkbox_change } = props;

    if repositories.is_empty() {
        html! {
            <p>{ "Loading…" }</p>
        }
    } else {
        repositories.iter()
                    .map(|repository: &Repository| {
            html! {
                <RepositoryCard repository={ repository.clone() } 
                                desired_archive_state={ archive_state_map.get_desired_state(repository.id) } 
                                {on_checkbox_change} />
            }
        }).collect()
    }
}
