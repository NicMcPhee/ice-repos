#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;

use crate::repository::{Repository, DesiredArchiveState};
use crate::components::repository_card::RepositoryCard;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub repositories: Vec<Repository>,
    pub on_checkbox_change: Callback<DesiredArchiveState>
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let Props { repositories, on_checkbox_change } = props;

    if repositories.is_empty() {
        html! {
            <p>{ "Loading…" }</p>
        }
    } else {
        repositories.iter()
                    .map(|repository: &Repository| {
            html! {
                <RepositoryCard repository={ repository.clone() } {on_checkbox_change} />
            }
        }).collect()
    }
}
