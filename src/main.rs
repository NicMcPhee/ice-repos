#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]

use std::num::ParseIntError;

use url::{Url, ParseError};

use reqwasm::http::Request;

use yew::prelude::*;
use yew_router::prelude::*;

use ice_repos::repository::Repository;
use ice_repos::components::{
    welcome::Welcome,
    about::About,
    organization_entry::OrganizationEntry,
    repository_list::RepositoryList,
};

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
pub struct RepositoryPaginatorProps {
    pub organization: String,
}

#[derive(Debug)]
pub struct RepositoryPaginatorState {
    // TODO: This should probably be an Option<Vec<Repository>> to distinguish between
    // an organization that has no repositories vs. we're waiting for repositories to
    // be loaded.
    repositories: Vec<Repository>,
    current_page: usize,
    last_page: usize,
}

#[derive(Debug)]
enum LinkParseError {
    InvalidUrl(ParseError),
    PageEntryMissing(Url),
    InvalidPageNumber(ParseIntError)
}

impl From<ParseError> for LinkParseError {
    fn from(e: ParseError) -> Self {
        Self::InvalidUrl(e)
    }
}

impl From<ParseIntError> for LinkParseError {
    fn from(e: ParseIntError) -> Self {
        Self::InvalidPageNumber(e)
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
        .ok_or_else(|| LinkParseError::PageEntryMissing(last_url.clone()))?;
    // This fails and returns a LinkParseError::PageNumberParseError if for some
    // reason the `num_pages_str` can't be parsed to a `usize`. This would also
    // presumably be an error or major API change on the part of GitHub.
    Ok(Some(num_pages_str.parse::<usize>()?))
}

// The GitHub default is 30; they allow no more than 100.
const REPOS_PER_PAGE: u8 = 7;

// * Convert the state back to &str to avoid all the copying.
//   * Maybe going to leave this alone? We got into a lot of lifetime issues that I didn't
//     want to deal with right now., because with the current version of Yew (v19), we can't
//     add generics to function components, and we'd need a lifetime component on the
//     properties, which bleeds through to the function component.
//   * Generics on function components have been added in the next version of Yew, so
//     we can come back to this if/when I upgrade to the newer version.


// This component has gotten _really_ long. At a minimum it should be moved
// into its own file. It's also possible that it should be converted into
// a struct component to help avoid some of the function call/return issues
// in the error handling.
#[function_component(RepositoryPaginator)]
pub fn repository_paginator(props: &RepositoryPaginatorProps) -> Html {
    fn paginator_button_class(page_number: usize, current_page: usize) -> String {
        if page_number == current_page { "btn btn-active".to_string() } else { "btn".to_string() }
    }

    fn make_button_callback(page_number: usize, repository_paginator_state: UseStateHandle<RepositoryPaginatorState>) -> Callback<MouseEvent> {
        Callback::from(move |_| {
            // Only make a new state if the page_number is different than the current_page number.
            if page_number == repository_paginator_state.current_page { return }

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

    fn try_extract(link_str: &str, current_page: usize) -> Result<usize, LinkParseError> {
        let parse_result = parse_last_page(link_str)?
            .unwrap_or(current_page);
        Ok(parse_result)
    }

    fn handle_parse_error(err: &LinkParseError) {
        web_sys::window()
            .unwrap()
            .alert_with_message("There was an error contacting the GitHub server; please try again")
            .unwrap();
        web_sys::console::error_1(
            &format!("There was an error parsing the link field in the HTTP response: {:?}", err).into());
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
                let request_url = format!("/orgs/{organization}/repos?sort=pushed&direction=asc&per_page={REPOS_PER_PAGE}&page={current_page}");
                let response = Request::get(&request_url).send().await.unwrap();
                let link = response.headers().get("link");
                web_sys::console::log_1(&format!("The link element of the header was <{link:?}>.").into());
                let last_page = match link.as_deref() {
                    None => 1,
                    Some(link_str) => match try_extract(link_str, current_page) {
                        Ok(last_page) => last_page,
                        Err(err) => { handle_parse_error(&err); return }
                    }
                };
                // TODO: This seems fairly slow when there are a lot of repositories. My guess
                // is that parsing the huge pile of JSON we get back is at least part of the
                // problem. Switching to GraphQL would potentially help this by allowing us to
                // specify the exact info we need for each repository (which is a tiny subset of
                // what GitHub currently provides), which should greatly reduce the
                // size of the JSON package and the cost of the parsing.
                let repos_result: Vec<Repository> = response.json().await.unwrap();
                let repo_state = RepositoryPaginatorState {
                    repositories: repos_result,
                    current_page,
                    // I'm increasingly wondering if Yew contexts are the right way to handle all this.
                    last_page
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
                // It's possible that `html_nested` would be a useful tool here.
                // https://docs.rs/yew/latest/yew/macro.html_nested.html
                {(1..=repository_paginator_state.last_page).map(|page_number| {
                        html! {
                            <button class={ paginator_button_class(page_number, repository_paginator_state.current_page) }
                                    onclick={ make_button_callback(page_number, repository_paginator_state.clone()) }>
                                { page_number }
                            </button>
                        }
                    }).collect::<Html>()}
                </div>
            }
            // TODO: I don't like this .clone(), but passing references got us into lifetime hell.
            <RepositoryList repositories={ repository_paginator_state.repositories.clone() } />
        </>
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
