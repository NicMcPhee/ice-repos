#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]

use std::collections::HashMap;
use std::num::ParseIntError;

use chrono::DateTime;
use chrono::Local;

use serde::Deserialize;
use serde_json::Value;

use url::{Url, ParseError};

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
pub struct OrganizationEntryProps {
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
//     want to deal with right now., because with the current version of Yew (v19), we can't
//     add generics to function components, and we'd need a lifetime component on the
//     properties, which bleeds through to the function component.
//   * Generics on function components have been added in the next version of Yew, so
//     we can come back to this if/when I upgrade to the newer version.
// * Deal with paging from GitHub

/// Controlled Text Input Component
#[function_component(OrganizationEntry)]
pub fn organization_entry(props: &OrganizationEntryProps) -> Html {
    let field_contents = use_state(|| String::from(""));

    let OrganizationEntryProps { on_submit } = props.clone();

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
pub struct RepositoryPaginatorProps {
    pub organization: String,
}

#[derive(Debug)]
pub struct RepositoryPaginatorState {
    repositories: Vec<Repository>,
    current_page: usize,
    last_page: usize,
}

// Things to work on, 13 August 2022
//  * Fix the problem with my regex for the pagination, probably by replacing the regex.
//  * Do something sensible about error handling
//    * Possibly introduce anyhow, eyre, or error_stack
//  * Turn list of repositories into a checkbox list

#[derive(Debug)]
enum LinkParseError {
    UrlParseError(ParseError),
    PageEntryMissingError(Url),
    PageNumParseError(ParseIntError)
}

impl From<ParseError> for LinkParseError {
    fn from(e: ParseError) -> Self {
        Self::UrlParseError(e)
    }
}

impl From<ParseIntError> for LinkParseError {
    fn from(e: ParseIntError) -> Self {
        Self::PageNumParseError(e)
    }
}

/*
 * This parses the `last` component of the link field in the response header from
 * GitHub, which tells us how many pages there are.
 * 
 * The link field looks like:
 * 
 * <https://api.github.com/organizations/18425666/repos?page=1&per_page=5>; rel="prev", <https://api.github.com/organizations/18425666/repos?page=3&per_page=5>; rel="next", <https://api.github.com/organizations/18425666/repos?page=5&per_page=5>; rel="last", <https://api.github.com/organizations/18425666/repos?page=1&per_page=5>; rel="first"
 */
fn parse_last_page(link_str: &str) -> Result<Option<usize>, LinkParseError> {
    // This split won't do the desired thing if there can ever be a comma in a
    // URL, but that doesn't seem likely given the structure of these GitHub URLs.
    let last_entry = link_str
        .split(", ")
        .find_map(|s| s.trim().strip_suffix(r#"; rel="last""#));
    // rel="last" is missing if we're on the last page
    let last_entry = match last_entry {
        None => return Ok(None),
        Some(s) => s
    };
    // This fails and returns a LinkParseError::UrlParseError if we can't parse the URL.
    let last_url = last_entry.trim_start_matches('<')
        .trim_end_matches('>')
        .parse::<Url>()?;
    let num_pages_str = last_url.query_pairs()
        // This returns the None variant if there was no "page" query parameter.
        // This is an error on GitHub's part (or a major change to their API),
        // and we'll return a LinkParseError::PageEntryMissingError if it happens.
        .find(|(k, _)| k.eq("page"))
        .map(|(_, v)| v)
        // The following line fails because of an ownership issue. I think that
        // we're passing ownership of into the closure, but num_pages_str still
        // points into that and we need to use that in the line after. I suspect
        // I'll need to clone something.
        .ok_or_else(|| LinkParseError::PageEntryMissingError(last_url))?;
    // This fails and returns a LinkParseError::PageNumberParseError if for some
    // reason the `num_pages_str` can't be parsed to a `usize`. This would also
    // presumably be an error or major API change on the part of GitHub.
    Ok(Some(num_pages_str.parse::<usize>()?))
}

/*
 * To make pagination work we'll need to:
 *   - Parse the total number of pages
 *   - Emit that back to the parent component
 *   - Display (with DaisyUI) the pagination controls
 *   - Send in the desired page through the props
 *   - Request the correct page
 */

#[derive(Clone, PartialEq, Properties)]
struct RepositoryListProps {
    repositories: Vec<Repository>
}

#[function_component(RepositoryList)]
fn repository_list(props: &RepositoryListProps) -> Html {
    let RepositoryListProps { repositories } = props;
    if repositories.is_empty() {
        html! {
            <p>{ "Loading…" }</p>
        }
    } else {
        repositories.iter()
                    .map(|repository: &Repository| {
            html! {
                <div>
                    if repository.archived {
                        <h2 class="text-2xl text-gray-300">{ repository.name.clone() }</h2>
                    } else {
                        <h2 class="text-2xl">{ repository.name.clone() }</h2>
                    }
                    {
                        repository.description.as_ref().map_or_else(
                            || html! { <p class="text-blue-700">{ "There was no description for this repository "}</p> },
                            |s| html! { <p class="text-green-700">{ s.clone() }</p> }
                        )
                    }
                    <p>{ format!("Last updated on {}", repository.updated_at.clone().format("%Y-%m-%d")) }</p>
                    <p>{ format!("Last pushed to on {}", repository.pushed_at.clone().format("%Y-%m-%d")) }</p>
                </div>
            }
        }).collect()
    }
}

#[function_component(RepositoryPaginator)]
pub fn repository_paginator(props: &RepositoryPaginatorProps) -> Html {
    fn paginator_button_class(page_number: usize, current_page: usize) -> String {
        if page_number == current_page { "btn btn-active".to_string() } else { "btn".to_string() }
    }

    fn make_button_callback(page_number: usize, repository_paginator_state: UseStateHandle<RepositoryPaginatorState>) -> Callback<MouseEvent> {
        Callback::from(move |_| {
            let repo_state = RepositoryPaginatorState {
                repositories: vec![],
                current_page: page_number,
                last_page: repository_paginator_state.last_page
            };
            web_sys::console::log_1(&format!("make_button_callback called with page number {page_number}.").into());
            web_sys::console::log_1(&format!("New state is {repo_state:?}.").into());
            repository_paginator_state.set(repo_state);
        })
    }

    let RepositoryPaginatorProps { organization } = props;
    web_sys::console::log_1(&format!("RepositoryPaginator called with organization {organization}.").into());
    let repository_paginator_state = use_state(|| RepositoryPaginatorState {
        repositories: vec![],
        current_page: 1,
        last_page: 0 // This is "wrong" and needs to be set after we've gotten our response.
    });
    {
        let repository_paginator_state = repository_paginator_state.clone();
        let organization = organization.clone();
        let current_page = repository_paginator_state.current_page;
        use_effect_with_deps(move |(organization, current_page)| {
            web_sys::console::log_1(&format!("use_effect_with_deps called with organization {organization}.").into());
            let organization = organization.clone();
            let current_page = *current_page;
            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&format!("spawn_local called with organization {organization}.").into());
                let request_url = format!("/orgs/{organization}/repos?sort=pushed&direction=asc&per_page=5&page={current_page}");
                let response = Request::get(&request_url).send().await.unwrap();
                let link = response.headers().get("link");
                web_sys::console::log_1(&format!("The link element of the header was <{link:?}>.").into());
                let repos_result: Vec<Repository> = response.json().await.unwrap();
                let repo_state = RepositoryPaginatorState {
                    repositories: repos_result,
                    current_page: 1,
                    last_page: link.as_deref().map_or(1, parse_last_page)
                };
                web_sys::console::log_1(&format!("The new repo state is <{repo_state:?}>.").into());
                repository_paginator_state.set(repo_state);
            });
            || ()
        }, (organization, current_page));
    }

    html! {
        <>
            if repository_paginator_state.last_page > 1 {
                <div class="btn-group">
                {
                    // Not sure why we need the containing pair of curly braces, but
                    // it's probably because we're inside an `html!` macro call. I
                    // might be able to remove the outer `html!` and add the `RepositoryList`
                    // component call to this iterator in some fashion.
                    //
                    // It's possible that `html_nested` would be a useful tool here.
                    // https://docs.rs/yew/latest/yew/macro.html_nested.html
                    (1..=repository_paginator_state.last_page).map(|page_number| {
                        html! {
                            <button class={ paginator_button_class(page_number, repository_paginator_state.current_page) }
                                    onclick={ make_button_callback(page_number, repository_paginator_state.clone()) }>
                                { page_number }
                            </button>
                        }
                    }).collect::<Html>()
                }
                </div>
            }
            // TODO: I don't like this .clone(), but passing references got us into lifetime hell.
            <RepositoryList repositories={ repository_paginator_state.repositories.clone() } />
        </>
    }
}

#[function_component(Welcome)]
fn welcome() -> Html {
    html! {
        <div class="text-center lg:text-left">
            <h1 class="text-5xl font-bold">{ "Welcome to" }</h1>
            <h1 class="text-5xl font-bold">{ "ice-repos!" }</h1>
            <p class="py-6">{ "A tool for archiving groups of GitHub repos" }</p>
        </div>
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let organization = use_state(|| String::from(""));

    let on_submit: Callback<String> = {
        let organization = organization.clone();
        Callback::from(move |string| { 
            organization.set(string);
            // web_sys::console::log_1(&format!("We got <{string}> from the text input!").into()) 
        })
    };

    html! {
        <div class="grid grid-cols-1 divide-y flex flex-col space-y-8">
            <div class="hero min-h-fit bg-base-200">
                <div class="hero-content flex-col lg:flex-row">
                    <Welcome />
                    <OrganizationEntry {on_submit} />
                </div>
            </div>

            // Where the list of repositories go
            if !organization.is_empty() {
                <div>
                    <h2 class="text-2xl">{ format!("The list of repositories for the organization {}", (*organization).clone()) }</h2>
                    <RepositoryPaginator key={(*organization).clone()} organization={(*organization).clone()} />
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
