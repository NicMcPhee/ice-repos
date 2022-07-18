use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;

use reqwasm::http::Request;
use reqwasm::http::Response;
use reqwasm::Error;

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
    pub on_change: Callback<String>,
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

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let field_contents = use_state(|| String::from(""));

    let TextInputProps { on_change } = props.clone();

    let oninput = {
        let field_contents = field_contents.clone();
        Callback::from(move |input_event: InputEvent| {
            field_contents.set(get_value_from_input_event(input_event))
        })
    };

    let onclick: Callback<MouseEvent> = {
        let field_contents = field_contents.clone();
        Callback::from(move |_| {
            on_change.emit((*field_contents).clone());
        })
    };

    html! {
        <div class="flex space-x-1">
            <input class="flex-auto w-64 bg-gray-600" type="text" {oninput} value={ (*field_contents).clone() } size=40 placeholder="Enter an organization name here" />
            <button {onclick} class="bg-gray-800 flex-none p-4">{ "Submit" }</button>
        </div>
    }
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct Repository {
    // id: usize,
    // name: String,
    // description: String,

    #[serde(flatten)]
    extras: HashMap<String, Value>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct RepositoryListProps {
    pub organization: String,
}

#[function_component(RepositoryList)]
pub fn repository_list(props: &RepositoryListProps) -> Html {
    let repositories = use_state(|| vec![]);
    {
        let repositories = repositories.clone();
        use_effect_with_deps(move |_| {
            let repositories = repositories.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // https://api.github.com/orgs/UMM-CSci-3601/repos
                let request_url = format!("/orgs/{org}/repos", 
                                org="UMM-CSci-3601");
                web_sys::console::log_1(&format!("Request URL = {}", request_url).into());
                let response = Request::get(&request_url).send().await.unwrap();
                // let repos_result: Value = response.json().await.unwrap();
                let repos_result: Vec<Repository> = response.json().await.unwrap();
                web_sys::console::log_1(&format!("Users = {:?}", repos_result).into());
                repositories.set(repos_result);
            });
            || ()
        }, ());
    }

    // html! {
    //     <p>{ format!("The result of the request was: {}", *repositories) }</p>
    // }
    repositories.iter().map(|repository: &Repository| {
        html! {
            <div>
                <h2>{ format!("{} ({})", repository.extras.get("name").unwrap().clone(), 
                                         repository.extras.get("id").unwrap())
                }</h2>
                <p class="text-green-300">{ 
                    repository.extras.get("description").unwrap().clone() 
                }</p>
            </div>
        }
    }).collect()
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let organization = use_state(|| String::from(""));

    let on_change: Callback<String> = {
        let organization = organization.clone();
        Callback::from(move |string| { 
            organization.set(string)
            // web_sys::console::log_1(&format!("We got <{}> from the text input!", string).into()) 
        })
    };

    html! {
        <div class="grid grid-cols-1 divide-y flex flex-col space-y-8">
            <div>
                <p class="text-4xl">{ "Welcome to Ice Repos" }</p> 
                <p class="text-2xl">{ "A tool for archiving groups of GitHub repos" }</p> 
            </div>

            <div>
                <p>{ "Enter either an organization or a GitHub Classroom"}</p>
                <TextInput {on_change} />
            </div>

            // Where the list of repositories go
            if !organization.is_empty() {
                <div>
                    <h2 class="text-2xl">{ format!("The list of repositories for the organization {}", (*organization).clone()) }</h2>
                    <RepositoryList organization={(*organization).clone()} />
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
