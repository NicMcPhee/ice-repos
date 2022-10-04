use gloo::console::log;
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

    log!(format!("We're in repo list with repo IDs {repo_ids:?}"));
    log!(format!("We're in repo list with ArchiveStateMap {archive_state_map:?}"));

    #[allow(clippy::option_if_let_else)]
    if let Some(repo_ids) = repo_ids {
        repo_ids.iter()
                .map(|repo_id: &RepoId| {
            html! {
                <RepositoryCard repository={ archive_state_map.get_repo(*repo_id).clone() } 
                                desired_archive_state={ archive_state_map.get_desired_state(*repo_id) } 
                                {on_checkbox_change} />
            }
        }).collect()
    } else {
        html! {
            <p>{ empty_repo_list_message }</p>
        }
    }
}
