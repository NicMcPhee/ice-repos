#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::collections::HashMap;

use chrono::DateTime;
use chrono::Local;

use serde::Deserialize;
use serde_json::Value;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;

use reqwasm::http::Request;

use yew::prelude::*;
use yew_router::prelude::*;

// ===================================================================================
// for {username}.github.io/{repo_name}
// replace 'yew-template-for-github.io' to your repo name

#[derive(Clone, Routable, PartialEq)]
enum RootRoute {
    #[at("/ice-repos/")]
    Home,
    #[at("/ice-repos/:s")]
    Route,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/ice-repos/about")]
    About,
    #[not_found]
    #[at("/ice-repos/404")]
    NotFound,
}

fn root_route(routes: &RootRoute) -> Html {
    match routes {
        RootRoute::Home => html! { <HomePage/> },
        RootRoute::Route => html! {
            <Switch<Route> render={Switch::render(switch)} />
        },
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::About => html! { <About/> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct TextInputProps {
    pub on_submit: Callback<String>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

// * Change the state when the text area loses focus instead of requiring a click on the
//   submit button.
//   * There is an `onfocusout` event that we should be able to leverage.
//     * This will trigger when we tab out, but I'm thinking that might be OK since there's
//       nowhere else to go in this simple interface.
//   * There's an `onsubmit` event. Would that be potentially useful?
// * Allow the user to press "Enter" instead of having to click on "Submit"
// * Convert the state back to &str to avoid all the copying.
//   * Maybe going to leave this alone? We got into a lot of lifetime issues that I didn't
//     want to deal with right now.
// * Deal with paging from GitHub

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let field_contents = use_state(|| String::from(""));

    let TextInputProps { on_submit } = props.clone();

    let oninput = {
        let field_contents = field_contents.clone();
        Callback::from(move |input_event: InputEvent| {
            field_contents.set(get_value_from_input_event(input_event));
        })
    };

    let onclick: Callback<MouseEvent> = {
        let field_contents = field_contents.clone();
        Callback::from(move |_| {
            on_submit.emit((*field_contents).clone());
        })
    };

    html! {
        <div class="hero min-h-fit bg-base-200">
            <div class="hero-content flex-col lg:flex-row">
                <div class="text-center lg:text-left">
                <h1 class="text-5xl font-bold">{ "Welcome to" }</h1>
                <h1 class="text-5xl font-bold">{ "ice-repos!" }</h1>
                <p class="py-6">{ "A tool for archiving groups of GitHub repos" }</p>
                </div>
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
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct Repository {
    id: usize,
    name: String,
    description: Option<String>,
    archived: bool,
    updated_at: DateTime<Local>,
    pushed_at: DateTime<Local>,

    #[serde(flatten)]
    extras: HashMap<String, Value>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct RepositoryListProps {
    pub organization: String,
}

// Things to work on, 30 July 2022
//  * Do something sensible about error handling
//  * Turn list of repositories into a checkbox list

// Do something about paging.

#[function_component(RepositoryList)]
pub fn repository_list(props: &RepositoryListProps) -> Html {
    let RepositoryListProps { organization } = props;
    web_sys::console::log_1(&format!("RepositoryList called with organization {}.", organization).into());
    let repositories = use_state(Vec::new);
    {
        let repositories = repositories.clone();
        let organization = organization.clone();
        use_effect_with_deps(move |organization| {
            web_sys::console::log_1(&format!("use_effect_with_deps called with organization {}.", organization).into());
            let organization = organization.clone();
            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&format!("spawn_local called with organization {}.", organization).into());
                let request_url = format!("/orgs/{org}/repos?sort=pushed&direction=asc", 
                                                    org=organization);
                let response = Request::get(&request_url).send().await.unwrap();
                let repos_result: Vec<Repository> = response.json().await.unwrap();
                repositories.set(repos_result);
            });
            || ()
        }, organization);
    }

    if repositories.is_empty() {
        html! {
            <p>{ "Loading…" }</p>
        }
    } else {
        repositories.iter()
                    .filter(|repository| { !repository.archived })
                    .map(|repository: &Repository| {
            html! {
                <div>
                    <h2 class="text-2xl">{ repository.name.clone() }</h2>
                    if let Some(description) = &repository.description {
                        <p class="text-green-700">{ 
                            description.clone() 
                        }</p>
                    } else {
                        <p class="text-blue-700">{
                            "There was no description for this repository"
                        }</p>
                    }
                    <p>{ format!("Last updated on {}", repository.updated_at.clone().format("%Y-%m-%d")) }</p>
                    <p>{ format!("Last pushed to on {}", repository.pushed_at.clone().format("%Y-%m-%d")) }</p>
                </div>
            }
        }).collect()
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let organization = use_state(|| String::from(""));

    let on_submit: Callback<String> = {
        let organization = organization.clone();
        Callback::from(move |string| { 
            organization.set(string);
            // web_sys::console::log_1(&format!("We got <{}> from the text input!", string).into()) 
        })
    };

    html! {
        <div class="grid grid-cols-1 divide-y flex flex-col space-y-8">
            <TextInput {on_submit} />

            // Where the list of repositories go
            if !organization.is_empty() {
                <div>
                    <h2 class="text-2xl">{ format!("The list of repositories for the organization {}", (*organization).clone()) }</h2>
                    <RepositoryList key={(*organization).clone()} organization={(*organization).clone()} />
                </div>
            }

            <div>
                <About/>
            </div>
        </div>
    }
}

#[function_component(About)]
fn about() -> Html {
    html! {
        <div class="mt-4">
            <p>{ "Explain the basic idea of the app here" }</p>
        </div>
    }
}

// ===================================================================================

/// main root
#[function_component(App)]
fn app() -> Html {
    html! {
        // ********************************************************
        // **    basename is not supported on yew 0.19.0 yet.    **
        // <BrowserRouter basename="/ice-repos/">
        //     <Switch<Route> render={Switch::render(switch)} />
        // </BrowserRouter>
        // ********************************************************
        <BrowserRouter>
            <Switch<RootRoute> render={Switch::render(root_route)} />
        </BrowserRouter>
    }
}

/// entry point
fn main() {
    yew::start_app::<App>();
}
