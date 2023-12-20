use yew::prelude::*;
use yew_router::prelude::use_navigator;
use yewdux::prelude::use_store;
use yewdux::{use_store_value, Store};

use crate::components::repository_card::ToggleState;
use crate::components::repository_list::RepositoryList;
use crate::page_repo_map::PageNumber;
use crate::repository::{ArchiveState, Organization};
use crate::Route;

const INCREMENT: PageNumber = 5;

#[derive(Clone, Debug, PartialEq, Default, Store)]
pub struct State {
    page: PageNumber,
}

impl State {
    pub fn reset(&mut self) {
        self.page = 0;
    }
}

fn prev_button_class(current_page: PageNumber) -> String {
    let mut class = "btn btn-primary".to_string();
    if current_page <= 1 {
        class.push_str(" btn-disabled");
    }
    class
}

fn next_button_class(loaded: bool) -> String {
    let mut class = "btn btn-primary".to_string();
    if !loaded {
        class.push_str(" btn-disabled");
    }
    class
}
#[function_component(RepositoryPaginator)]
pub fn repository_paginator() -> Html {
    let (state, dispatch) = use_store::<State>();
    let org = use_store_value::<Organization>();
    let total_repos = org.repositories.len();
    let max_pages = total_repos / INCREMENT;
    let filter = ArchiveState::filter_select();
    let toggle_state = ToggleState {
        on: ArchiveState::Archive,
        off: ArchiveState::Keep,
    };
    let range = state.page..state.page.saturating_add(INCREMENT);
    let prev = dispatch.reduce_mut_callback(|state| {
        state.page = state.page.saturating_sub(INCREMENT);
    });
    let history = use_navigator().unwrap();
    let next_btn = if state.page >= max_pages * INCREMENT {
        let onclick = Callback::from(move |_: MouseEvent| history.push(&Route::ReviewAndSubmit));
        html! {
            <button class={ next_button_class(true) } {onclick}>
                { "Review & Submit" }
            </button>
        }
    } else {
        let onclick = dispatch.reduce_mut_callback(move |state| {
            state.page = state
                .page
                .saturating_add(INCREMENT)
                .min(max_pages * INCREMENT);
        });
        html! {
            <button class={ next_button_class(true) } {onclick}>
                { "Next" }
                // { if current_page == last_page { "Review & Submit" } else { "Next" } }
            </button>
        }
    };

    html! {
        <>
            <RepositoryList {range} {filter} {toggle_state} empty_repo_list_message={ "Loading..." } />
            <div class="btn-group">
                <button class={ prev_button_class(state.page / INCREMENT + 1) } onclick={prev}>
                    { "Prev" }
                    </button>
                <button class="btn btn-active" disabled=true>
                    { format!("{}/{}", state.page / INCREMENT + 1, max_pages + 1) }
                </button>
                { next_btn }
            </div>
        </>
    }
}
