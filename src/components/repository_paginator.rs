#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::num::ParseIntError;
use std::rc::Rc;

use url::{Url, ParseError};

use reqwasm::http::{Request};

use yew::prelude::*;
use yewdux::prelude::{use_store, Dispatch};

use crate::repository::{Repository, DesiredArchiveState, Organization, ArchiveStateMap};
use crate::components::repository_list::RepositoryList;

// TODO: Idea from @esitsu@Twitch is to wrap the State with either
//   a Mutex or a RwLock so that we can directly modify the elements
//   of the State. This means we don't have to call `.set()` to update
//   the component state, and we might avoid some cloning as a result.

#[derive(Debug)]
struct State {
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
const REPOS_PER_PAGE: u8 = 3;

fn paginator_button_class(page_number: usize, current_page: usize) -> String {
    if page_number == current_page { "btn btn-active".to_string() } else { "btn".to_string() }
}

fn make_button_callback(page_number: usize, repository_paginator_state: UseStateHandle<State>) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        // Only make a new state if the page_number is different than the current_page number.
        if page_number == repository_paginator_state.current_page { return }

        let repo_state = State {
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
    #[allow(clippy::unwrap_used)]
    web_sys::window()
        .unwrap()
        .alert_with_message("There was an error contacting the GitHub server; please try again")
        .unwrap();
    web_sys::console::error_1(
        &format!("There was an error parsing the link field in the HTTP response: {:?}", err).into());
}

fn update_state_for_organization(organization: Rc<Organization>, archive_state_dispatch: Dispatch<ArchiveStateMap>, current_page: usize, state: UseStateHandle<State>) {
    wasm_bindgen_futures::spawn_local(async move {
        assert!(organization.name.is_some());
        // This unwrap() should be safe because the `RepositoryPaginator` is only rendered in
        // `HomePage` if the organization is a `Some` variant.
        #[allow(clippy::unwrap_used)]
        let organization = organization.name.as_ref().unwrap();

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
        archive_state_dispatch.reduce_mut(|archive_state_map| {
            archive_state_map.with_repos(&repos_result);
        });
        let repo_state = State {
            repositories: repos_result,
            current_page,
            // I'm increasingly wondering if Yew contexts are the right way to handle all this.
            last_page
        };
        web_sys::console::log_1(&format!("The new repo state is <{repo_state:?}>.").into());
        state.set(repo_state);
    });
}

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
pub fn repository_paginator() -> Html {
    let (organization, _) = use_store::<Organization>();
    let (_, archive_state_dispatch) = use_store::<ArchiveStateMap>();

    web_sys::console::log_1(&format!("RepositoryPaginator called with organization {:?}.", organization.name).into());

    let repository_paginator_state = use_state(|| State {
        repositories: vec![],
        current_page: 1,
        last_page: 0 // This is "wrong" and needs to be set after we've gotten our response.
    });
    {
        let repository_paginator_state = repository_paginator_state.clone();
        let current_page = repository_paginator_state.current_page;
        let archive_state_dispatch = archive_state_dispatch.clone();
        use_effect_with_deps(
            move |(organization, current_page)| {
                update_state_for_organization(organization.clone(), archive_state_dispatch, *current_page, repository_paginator_state);
                || ()
            }, 
            (organization, current_page)
        );
    }
    
    let on_checkbox_change: Callback<DesiredArchiveState> = {
        Callback::from(move |desired_archive_state| {
            let DesiredArchiveState { id, desired_archive_state } = desired_archive_state;
            archive_state_dispatch.reduce_mut(|archive_state_map| {
                archive_state_map.update_desired_state(id, desired_archive_state);
            });
        })
    };
    
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
            <RepositoryList repositories={ repository_paginator_state.repositories.clone() }
                            {on_checkbox_change} />
        </>
    }
}