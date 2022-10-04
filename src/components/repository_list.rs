use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::repository::{RepoId, DesiredArchiveState, ArchiveStateMap};
use crate::components::repository_card::RepositoryCard;

// TODO: Can we use `AttrValue` instead of `String` here?
// `AttrValue` is supposed to be more efficient
// because cloning `String`s can be expensive.
// https://yew.rs/docs/concepts/components/properties#memoryspeed-overhead-of-using-properties
#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub repo_ids: Option<Vec<RepoId>>,
    pub empty_repo_list_message: String,
    pub on_checkbox_change: Callback<DesiredArchiveState>
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let Props { repo_ids, 
                empty_repo_list_message, 
                on_checkbox_change } = props;

    let (archive_state_map, _) = use_store::<ArchiveStateMap>();

    if repositories.is_empty() {
        html! {
            <p>{ empty_repo_list_message }</p>
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
