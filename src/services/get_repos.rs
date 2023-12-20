use std::error::Error;

use reqwasm::http::Request;
use yewdux::Dispatch;

use crate::{
    page_repo_map::PageNumber,
    repository::{ArchiveState, Organization, Repository, RepositoryInfo},
};

// The GitHub default is 30; they allow no more than 100.
const REPOS_PER_PAGE: u8 = 30;

async fn load_page(
    organization: &str,
    current_page: PageNumber,
) -> Result<Vec<RepositoryInfo>, Box<dyn Error>> {
    let request_url = format!(
        "https://api.github.com/orgs/{organization}/repos?sort=pushed&direction=asc&per_page={REPOS_PER_PAGE}&page={current_page}"
    );
    let response = Request::get(&request_url).send().await?;
    Ok(response.json().await?)
}

pub fn load_organization(organization: &str, dispatch: Dispatch<Organization>) {
    let organization = organization.to_owned();
    yew::platform::spawn_local(async move {
        let mut page = 1;
        while let Ok(repos) = load_page(&organization, page).await {
            if repos.is_empty() {
                break;
            }

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

            page += 1;
        }
    });
}
