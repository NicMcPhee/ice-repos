use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;
use yewdux::use_store;

use crate::organization::{ArchiveState, Organization, RepoId};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    /// The ID of the repository to display.
    pub repo_id: RepoId,
    /// The state to set the repository to when the checkbox is checked/unchecked.
    pub toggle_state: ToggleState,
}

/// The toggle state for a repository card. Decsribes what ArchiveState to set when the checkbox is
/// checked/unchecked.
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct ToggleState {
    pub on: ArchiveState,
    pub off: ArchiveState,
}

#[function_component(RepositoryCard)]
pub fn repository_card(props: &Props) -> Html {
    let (org, org_dispatch) = use_store::<Organization>();
    let Props {
        repo_id,
        toggle_state,
    } = *props;

    let repo = match org.repositories.get(&repo_id) {
        Some(repo) => repo,
        None => return Default::default(),
    };

    let onclick = org_dispatch.reduce_mut_callback_with(move |state, mouse_event: MouseEvent| {
        let Some(repo) = state.repositories.get_mut(&repo_id) else {
            return;
        };

        repo.archive_state = if is_checked(&mouse_event) {
            toggle_state.on
        } else {
            toggle_state.off
        };
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

fn is_checked(mouse_event: &MouseEvent) -> bool {
    mouse_event
        .target()
        .unwrap_throw()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap_throw()
        .checked()
}
