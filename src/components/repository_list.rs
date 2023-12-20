use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::components::repository_card::RepositoryCard;
use crate::repository::{ArchiveState, Organization};

use super::repository_card::ToggleState;

// TODO: Can we use `AttrValue` instead of `String` here?
// `AttrValue` is supposed to be more efficient
// because cloning `String`s can be expensive.
// https://yew.rs/docs/concepts/components/properties#memoryspeed-overhead-of-using-properties
#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub range: std::ops::Range<usize>,
    pub filter: Vec<ArchiveState>,
    pub toggle_state: ToggleState,
    pub empty_repo_list_message: String,
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let (org, _) = use_store::<Organization>();

    let Props {
        range,
        filter,
        empty_repo_list_message,
        toggle_state,
    } = props;

    let mut repo_ids = org
        .repositories
        .iter()
        .skip(range.start)
        .filter(|(_, repo)| filter.contains(&repo.archive_state))
        .take(range.end - range.start)
        .map(|(repo_id, _)| repo_id)
        .peekable();
    // Check if we have any repos to display.
    let is_empty = repo_ids.peek().is_none();
    let repos_view = repo_ids
        .map(|repo_id| {
            let toggle_state = *toggle_state;
            html! {
                <RepositoryCard {repo_id} {toggle_state} />
            }
        })
        .collect();

    if !is_empty {
        repos_view
    } else {
        html! {
            <p>{ empty_repo_list_message }</p>
        }
    }
}
