use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::use_store;

use crate::repository::{ArchiveState, Organization, RepoId};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // TODO: Having to clone the repository in `RepositoryList` is annoying and it
    // would be cool to turn this into a reference without making a mess of the
    // memory management.
    pub repo_id: RepoId,
    // If this is the None variant, then this repository should already be archived.
    // If it's a Some variant, then the enclosed boolean should indicate the desired
    // state for this repository.
    pub toggle_state: ToggleState,
}

#[derive(Clone, PartialEq, Copy)]
pub struct ToggleState {
    pub on: ArchiveState,
    pub off: ArchiveState,
}

#[function_component(RepositoryCard)]
pub fn repository_card(props: &Props) -> Html {
    let Props {
        repo_id,
        toggle_state,
    } = *props;

    let (org, org_dispatch) = use_store::<Organization>();
    let repo = match org.repositories.get(&repo_id) {
        Some(repo) => repo,
        None => return html! {},
    };

    let onclick = org_dispatch.reduce_mut_callback_with(move |state, mouse_event: MouseEvent| {
        let event_target = mouse_event.target().unwrap_throw();
        let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
        let checked = target.checked();

        if let Some(repo) = state.repositories.get_mut(&repo_id) {
            if checked {
                repo.archive_state = toggle_state.on;
            } else {
                repo.archive_state = toggle_state.off;
            }
        }
    });

    html! {
        <div class="card card-compact">
            <div class="card-body">
                if repo.info.archived {
                    <p class="italic">{ "This repository is already archived" }</p>
                } else {
                    <div class="card-actions">
                        <div class="form-control">
                            <label class="label cursor-pointer">
                                <input type="checkbox"
                                       checked={ repo.archive_state == toggle_state.on }
                                       class="checkbox" {onclick} />
                                <p class="label-text italic ml-2">{ "Archive this repository" }</p>
                            </label>
                        </div>
                    </div>
                }
                if repo.info.archived {
                    <h2 class="card-title text-gray-300">{ &repo.info.name }</h2>
                } else {
                    <h2 class="card-title">{ &repo.info.name }</h2>
                }
                {
                    repo.info.description.as_ref().map_or_else(
                        || html! { <p class="text-blue-700">{ "There was no description for this repository "}</p> },
                        |s| html! { <p class="text-green-700">{ s }</p> }
                    )
                }
                <p>{ format!("Last updated on {}; ", repo.info.updated_at.format("%Y-%m-%d")) }
                   { format!("last pushed to on {}", repo.info.pushed_at.format("%Y-%m-%d")) }</p>
            </div>
        </div>
    }
}
