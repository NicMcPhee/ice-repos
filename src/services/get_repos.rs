use std::{error::Error, num::ParseIntError};

use reqwasm::http::{Request, Response};
use yewdux::Dispatch;

use crate::{
    organization::{ArchiveState, Organization, Repository, RepositoryInfo},
    page_repo_map::PageNumber,
};

// The GitHub default is 30; they allow no more than 100.
const REPOS_PER_PAGE: u8 = 30;

/// Load the repositories for the given organization.
pub fn load_organization(organization: &str, dispatch: Dispatch<Organization>) {
    let organization = organization.to_owned();
    yew::platform::spawn_local(async move {
        if let Err(error) = load_organization_task(&organization, dispatch).await {
            web_sys::console::log_1(
                &format!("spawn_local called with organization {error:?}.").into(),
            );
        }
    });
}

pub async fn load_organization_task(
    organization: &str,
    dispatch: Dispatch<Organization>,
) -> Result<(), Box<dyn Error>> {
    let mut current_page = 1;

    let response = get_repos(organization, current_page).await?;
    append_repos(parse_repos(&response).await?, &dispatch);

    let last_page = response
        .headers()
        .get("link")
        .and_then(|link| parse_last_page(&link).transpose())
        .transpose()
        .map_err(|err| format!("Error parsing last page from link: {:?}", err))?
        .unwrap_or(current_page);
    current_page += 1;
    while current_page <= last_page {
        let response = get_repos(organization, current_page).await?;
        // This call causes the entire organization state to be cloned. This is probably fine,
        // but may not scale well for a very large repo list.
        append_repos(parse_repos(&response).await?, &dispatch);

        current_page += 1;
    }

    Ok(())
}

fn append_repos(repos: Vec<RepositoryInfo>, dispatch: &Dispatch<Organization>) {
    let it = repos.into_iter().map(|info| Repository {
        archive_state: if info.archived {
            ArchiveState::AlreadyArchived
        } else {
            ArchiveState::Archive
        },
        info,
    });

    dispatch.reduce_mut(move |org| {
        org.repositories.update(it);
    });
}

async fn get_repos(organization: &str, page: PageNumber) -> Result<Response, Box<dyn Error>> {
    let base = "https://api.github.com/orgs";
    let args = "sort=pushed&direction=asc";
    let request_url =
        format!("{base}/{organization}/repos?{args}&per_page={REPOS_PER_PAGE}&page={page}");

    Ok(Request::get(&request_url).send().await?)
}

async fn parse_repos(response: &Response) -> Result<Vec<RepositoryInfo>, Box<dyn Error>> {
    Ok(response.json().await?)
}

/// This parses the `last` component of the link field in the response header from
/// GitHub, which tells us how many pages there are.
///
/// The link field looks like:
///
/// <https://api.github.com/organizations/18425666/repos?page=1&per_page=5>; rel="prev", <https://api.github.com/organizations/18425666/repos?page=3&per_page=5>; rel="next", <https://api.github.com/organizations/18425666/repos?page=5&per_page=5>; rel="last", <https://api.github.com/organizations/18425666/repos?page=1&per_page=5>; rel="first"
///
fn parse_last_page(link_str: &str) -> Result<Option<PageNumber>, LinkParseError> {
    // This split won't do the desired thing if there can ever be a comma in a
    // URL, but that doesn't seem likely given the structure of these GitHub URLs.
    let last_entry = link_str
        .split(", ")
        .find_map(|s| s.trim().strip_suffix(r#"; rel="last""#));
    // rel="last" is missing if we're on the last page
    let last_entry = match last_entry {
        None => return Ok(None),
        Some(s) => s,
    };
    // This fails and returns a LinkParseError::UrlParseError if we can't parse the URL.
    let last_url = last_entry
        .trim_start_matches('<')
        .trim_end_matches('>')
        .parse::<url::Url>()?;
    let num_pages_str = last_url
        .query_pairs()
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

#[derive(Debug)]
enum LinkParseError {
    InvalidUrl(url::ParseError),
    PageEntryMissing(url::Url),
    InvalidPageNumber(ParseIntError),
}

impl From<url::ParseError> for LinkParseError {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidUrl(e)
    }
}

impl From<ParseIntError> for LinkParseError {
    fn from(e: ParseIntError) -> Self {
        Self::InvalidPageNumber(e)
    }
}
