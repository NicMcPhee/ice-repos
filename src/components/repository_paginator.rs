use std::num::ParseIntError;
use std::ops::Deref;

use url::{Url, ParseError};

use reqwasm::http::{Request};

use yew_router::prelude::*;
use yew::prelude::*;
use yewdux::prelude::{use_store, Dispatch};
use yewdux::store::Store;

use crate::Route;
use crate::repository::{Repository, DesiredArchiveState, DesiredStateMap, DesiredState};
use crate::page_repo_map::{PageRepoMap, PageNumber};
use crate::components::repository_list::RepositoryList;

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct Props {
    pub organization: String
}
#[derive(Debug, Clone)]
struct State {
    current_page: PageNumber,
    last_page: PageNumber
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
fn parse_last_page(link_str: &str) -> Result<Option<PageNumber>, LinkParseError> {
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
    // reason the `num_pages_str` can't be parsed to a `PageNumber`. This would also
    // presumably be an error or major API change on the part of GitHub.
    Ok(Some(num_pages_str.parse::<PageNumber>()?))
}

// The GitHub default is 30; they allow no more than 100.
const REPOS_PER_PAGE: u8 = 30;

fn prev_button_class(current_page: PageNumber) -> String {
    let mut class = "btn btn-primary".to_string();
    if current_page == 1 {
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

fn make_button_callback(page_number: PageNumber, repository_paginator_state: UseStateHandle<State>) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let repo_state = State {
            current_page: page_number,
            last_page: repository_paginator_state.last_page
        };
        web_sys::console::log_1(&format!("make_button_callback called with page number {page_number}.").into());
        web_sys::console::log_1(&format!("New state is {repo_state:?}.").into());
        repository_paginator_state.set(repo_state);
    })
}

fn try_extract(link_str: &str, current_page: PageNumber) -> Result<PageNumber, LinkParseError> {
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

fn load_new_page(organization: &str, desired_state_map_dispatch: Dispatch<DesiredStateMap>, page_map: UseStateHandle<PageRepoMap>, current_page: PageNumber, state: UseStateHandle<State>) {
    let organization = organization.to_owned();
    // TODO: Possibly change `spawn_local` to `use_async`.
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
        
        desired_state_map_dispatch.reduce_mut(|desired_state_map| {
            desired_state_map.with_repos(&repos_result);
        });

        let mut new_page_map 
            = page_map
                .deref()
                .clone();
        new_page_map.add_page(
            current_page,
            repos_result.iter().map(|r| r.id).collect()
        );
        page_map.set(new_page_map);

        let repo_state = State {
            current_page,
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
pub fn repository_paginator(props: &Props) -> Html {
    let Props { organization } = props;
    let page_map = use_state(|| PageRepoMap::new());

    // TODO: Change this from being a Yewdux global to being either "internal" state for
    //   the paginator, or use Yew's context tools to share this with the review and submit
    //   component.
    let (desired_state_map, desired_state_map_dispatch) = use_store::<DesiredStateMap>();
    {
        let organization = organization.clone();
        let desired_state_map_dispatch = desired_state_map_dispatch.clone();
        use_effect_with_deps(
            move |_| {
                desired_state_map_dispatch.set(DesiredStateMap::new());
                || ()
            },
            organization
        )
    }

    web_sys::console::log_1(&format!("RepositoryPaginator called with organization {:?}.", organization).into());
    web_sys::console::log_1(&format!("Current StateMap is {:?}.", desired_state_map).into());

    let repository_paginator_state = use_state(|| State {
        current_page: 1,
        last_page: 0
    });

    let State { current_page, last_page }
        = (*repository_paginator_state).clone();

    // TODO: We want to see if the current page has already been loaded, and only do
    // `update_state_for_organization` if it has not been loaded yet. Might make sense
    // to fix this along with switching to "Prev"/"Next" UI model.
    // TODO: It's possible that this would all be easier if we used a structural component
    //   here with messages for the various updates instead of having multiple `use_effect_with_deps`
    //   calls.
    {
        let organization = organization.clone();
        let page_map = page_map.clone();
        let repository_paginator_state = repository_paginator_state.clone();
        let desired_state_map_dispatch = desired_state_map_dispatch.clone();
        use_effect_with_deps(
            move |current_page| {
                let current_page = *current_page;
                if !page_map.has_loaded_page(current_page) {
                    load_new_page(&organization.clone(), 
                    desired_state_map_dispatch, 
                    page_map,
                    current_page, 
                    repository_paginator_state);
                }
                || ()
            }, 
            current_page
        );
    }
    
    let on_checkbox_change: Callback<DesiredArchiveState> = {
        Callback::from(move |desired_archive_state| {
            let DesiredArchiveState { id, desired_archive_state } = desired_archive_state;
            desired_state_map_dispatch.reduce_mut(|state_map| {
                state_map.update_desired_state(id, DesiredState::from_paginator_state(desired_archive_state));
            });
        })
    };

    let prev: Callback<MouseEvent> = {
        // assert!(current_page > 1);
        make_button_callback(current_page-1, repository_paginator_state.clone())
    };

    let next_or_review: Callback<MouseEvent> = {
        if current_page < last_page {
            make_button_callback(current_page+1, repository_paginator_state)
        } else {
            let history = use_history().unwrap();
            Callback::from(move |_: MouseEvent| history.push(Route::ReviewAndSubmit))
        }
    };
    
    html! {
        <>
            <RepositoryList repo_ids={page_map.get_repo_ids(current_page)}
                            empty_repo_list_message={ "Loading..." }
                            {on_checkbox_change} />
            <div class="btn-group">
                <button class={ prev_button_class(current_page) } onclick={prev}>
                    { "Prev" }
                    </button>
                <button class="btn btn-active" disabled=true>
                    { format!("{}/{}", current_page, last_page) }
                </button>
                <button class={ next_button_class(page_map.has_loaded_page(current_page)) } onclick={next_or_review}>
                    { if current_page == last_page { "Review & Submit" } else { "Next" } }
                </button>
            </div>
        </>
    }
}