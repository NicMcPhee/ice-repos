#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;

use crate::repository::{Repository, DesiredArchiveState};
use crate::components::repository_card::RepositoryCard;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub repositories: Vec<Repository>
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let Props { repositories } = props;

    let on_submit: Callback<DesiredArchiveState> = {
        Callback::from(move |desired_archive_state| { 
            // organization.set(string);
            let DesiredArchiveState { id, desired_archive_state } = desired_archive_state;
            web_sys::console::log_1(&format!("We clicked <{id}> with value {desired_archive_state}").into());
        })
    };

    if repositories.is_empty() {
        html! {
            <p>{ "Loading…" }</p>
        }
    } else {
        repositories.iter()
                    .map(|repository: &Repository| {
            html! {
                <RepositoryCard repository={ repository.clone() } on_checkbox_change={on_submit.clone()} />
            }
        }).collect()
    }
}
