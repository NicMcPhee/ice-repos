use yew::prelude::*;
use yew_router::prelude::use_navigator;
use yewdux::prelude::use_store;
use yewdux::{use_store_value, Store};

use crate::components::repository_card::ToggleState;
use crate::components::repository_list::RepositoryList;
use crate::organization::{ArchiveState, Organization, RepoFilter};
use crate::page_repo_map::PageNumber;
use crate::Route;

const PAGE_SIZE: PageNumber = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Store)]
pub struct State {
    page: PageNumber,
}

impl State {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[function_component(RepositoryPaginator)]
pub fn repository_paginator() -> Html {
    let (state, dispatch) = use_store::<State>();
    let org = use_store_value::<Organization>();
    let history = use_navigator().unwrap();

    let filter = RepoFilter::all();
    let total_repos = org.repositories.len();
    let total_pages = {
        let pages = total_repos / PAGE_SIZE;
        if total_repos % PAGE_SIZE == 0 {
            pages.saturating_sub(1)
        } else {
            pages
        }
    };
    let range = (state.page * PAGE_SIZE)..(state.page * PAGE_SIZE).saturating_add(PAGE_SIZE);
    let toggle_state = ToggleState {
        on: ArchiveState::Archive,
        off: ArchiveState::Keep,
    };

    let prev = dispatch.reduce_mut_callback(|state| {
        state.page = state.page.saturating_sub(1);
    });
    let next_btn = if state.page >= total_pages {
        let onclick = Callback::from(move |_: MouseEvent| history.push(&Route::ReviewAndSubmit));

        html! {
            <button class={ next_button_class(true) } {onclick}>
                { "Review & Submit" }
            </button>
        }
    } else {
        let onclick = dispatch.reduce_mut_callback(move |state| {
            state.page = state.page.saturating_add(1).min(total_pages);
        });
        let loaded = !org.repositories.is_empty();

        html! {
            <button class={ next_button_class(loaded) } {onclick}>
                { "Next" }
            </button>
        }
    };

    html! {
        <>
            <RepositoryList {range} {filter} {toggle_state} empty_repo_list_message={ "Loading..." } />
            <div class="btn-group">
                <button class={ prev_button_class(state.page + 1) } onclick={prev}>
                    { "Prev" }
                    </button>
                <button class="btn btn-active" disabled=true>
                    { format!("{}/{}", state.page + 1, total_pages + 1) }
                </button>
                { next_btn }
            </div>
        </>
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
