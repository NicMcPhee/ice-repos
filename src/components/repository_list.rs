use yew::prelude::*;
use yewdux::use_store_value;

use crate::components::repository_card::RepositoryCard;
use crate::organization::{Organization, RepoFilter};

use super::repository_card::ToggleState;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub range: std::ops::Range<usize>,
    pub filter: RepoFilter,
    pub toggle_state: ToggleState,
    pub empty_repo_list_message: AttrValue,
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &Props) -> Html {
    let org = use_store_value::<Organization>();
    let Props {
        range,
        filter,
        empty_repo_list_message,
        toggle_state,
    } = props;

    let mut repos = org.repositories.select(range.clone(), filter).peekable();

    let is_empty = repos.peek().is_none();
    if is_empty {
        return html! {
            <p>{ empty_repo_list_message }</p>
        };
    }

    repos
        .map(|repo| {
            let toggle_state = *toggle_state;
            html! {
                <RepositoryCard repo_id={repo.info.id} {toggle_state} />
            }
        })
        .collect()
}
