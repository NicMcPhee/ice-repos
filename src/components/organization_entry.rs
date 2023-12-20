use std::ops::Deref;

use wasm_bindgen::{JsCast, UnwrapThrowExt};

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{repository::Organization, services::get_repos::load_organization};

// * Change the state when the text area loses focus instead of requiring a click on the
//   submit button.
//   * There is an `onfocusout` event that we should be able to leverage.
//     * This will trigger when we tab out, but I'm thinking that might be OK since there's
//       nowhere else to go in this simple interface.
//   * There's an `onsubmit` event. Would that be potentially useful?
// * Allow the user to press "Enter" instead of having to click on "Submit"

/// Controlled Text Input Component
#[function_component(OrganizationEntry)]
pub fn organization_entry() -> Html {
    let field_contents = use_state(|| String::from(""));
    let org_dispatch = use_dispatch::<Organization>();
    let pagination_dispatch = use_dispatch::<crate::components::repository_paginator::State>();

    let oninput = {
        let field_contents = field_contents.clone();
        Callback::from(move |input_event: InputEvent| {
            field_contents.set(get_value_from_input_event(input_event));
        })
    };

    let onclick = {
        let field_contents = field_contents.clone();
        Callback::from(move |_| {
            if field_contents.is_empty() {
                return;
            }

            org_dispatch.reduce_mut(|org| {
                let name = field_contents.deref().clone().into();
                org.set_name(name);
            });

            pagination_dispatch.reduce_mut(|state| {
                state.reset();
            });

            load_organization(&field_contents, org_dispatch.clone());
        })
    };

    // TODO: Use a form so we can also submit on "Enter" instead of having to click on "Submit"
    html! {
        <div class="card flex-shrink-0 w-full max-w-sm shadow-2xl bg-base-100">
            <div class="card-body">
                <div class="form-control">
                <label class="label">
                    <span class="label-text">{ "What organization would you like to archive repositories for?" }</span>
                </label>
                <input type="text" placeholder="organization" class="input input-bordered" {oninput} value={ (*field_contents).clone() }/>
                </div>
                <div class="form-control mt-6">
                <button type="submit" class="btn btn-primary" {onclick}>{ "Submit" }</button>
                </div>
            </div>
        </div>
    }
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}
