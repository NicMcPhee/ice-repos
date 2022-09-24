use wasm_bindgen::{UnwrapThrowExt, JsCast};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::repository::{Repository, DesiredArchiveState};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // TODO: Having to clone the repository in `RepositoryList` is annoying and it
    // would be cool to turn this into a reference without making a mess of the
    // memory management.
    pub repository: Repository,
    // If this is the None variant, then this repository should already be archived.
    // If it's a Some variant, then the enclosed boolean should indicate the desired
    // state for this repository.
    pub desired_archive_state: Option<bool>,
    pub on_checkbox_change: Callback<DesiredArchiveState>
}

#[function_component(RepositoryCard)]
pub fn repository_card(props: &Props) -> Html {
    let Props { repository, desired_archive_state, on_checkbox_change } 
            = props;

    // If we pass this assertion, then the desired_archive_state.unwrap() in the HTML
    // should be safe.
    if desired_archive_state.is_none() {
        assert!(repository.archived);
    }

    let onclick: Callback<MouseEvent> = {
        let id = repository.id;
        let on_checkbox_change = on_checkbox_change.clone();

        Callback::from(move |mouse_event: MouseEvent| {
            let event_target = mouse_event.target().unwrap_throw();
            let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
            let desired_archive_state = target.checked();

            web_sys::console::log_1(&format!("In ME callback desired state is {desired_archive_state}.").into());

            on_checkbox_change.emit(DesiredArchiveState {
                id,
                desired_archive_state
            });
        })
    };

    html! {
        <div class="card card-compact">
            <div class="card-body">
                if repository.archived {
                    <p class="italic">{ "This repository is already archived" }</p>
                } else {
                    <div class="card-actions">
                        <div class="form-control">
                            <label class="label cursor-pointer">
                                <input type="checkbox" 
                                       checked={ #[allow(clippy::unwrap_used)] desired_archive_state.unwrap() } 
                                       class="checkbox" {onclick} />
                                <p class="label-text italic ml-2">{ "Archive this repository" }</p> 
                            </label>
                        </div>
                    </div>
                }
                if repository.archived {
                    <h2 class="card-title text-gray-300">{ &repository.name }</h2>
                } else {
                    <h2 class="card-title">{ &repository.name }</h2>
                }
                {
                    repository.description.as_ref().map_or_else(
                        || html! { <p class="text-blue-700">{ "There was no description for this repository "}</p> },
                        |s| html! { <p class="text-green-700">{ s }</p> }
                    )
                }
                <p>{ format!("Last updated on {}; ", repository.updated_at.format("%Y-%m-%d")) }
                   { format!("last pushed to on {}", repository.pushed_at.format("%Y-%m-%d")) }</p>
            </div>
        </div>
    }
}
