use web_sys::MouseEvent;
use yew::{function_component, html, Callback};
use yewdux::prelude::use_store;

use crate::repository::{ArchiveStateMap, DesiredArchiveState, ArchiveState};
use crate::components::repository_list::RepositoryList;
use crate::services::archive_repos::archive_repositories;

/// Review selected repositories to archive and
/// submit archive requests.
#[function_component(ReviewAndSubmit)]
pub fn review_and_submit() -> Html {
    let (archive_state_map, archive_state_dispatch) 
        = use_store::<ArchiveStateMap>();

    let on_checkbox_change: Callback<DesiredArchiveState> = {
        Callback::from(move |desired_archive_state| {
            let DesiredArchiveState { id, desired_archive_state } = desired_archive_state;
            archive_state_dispatch.reduce_mut(|archive_state_map| {
                archive_state_map.update_desired_state(id, ArchiveState::from_review_state(desired_archive_state));
            });
        })
    };

    let onclick: Callback<MouseEvent> = {
        let archive_state_map = archive_state_map.clone();
        Callback::from(move |_| {
            archive_repositories(archive_state_map.get_repos_to_archive());
        })
    };

    // TODO: We need some kind of shared header that comes across to pages like this.
    html! {
        <div>
            <RepositoryList repo_ids={ archive_state_map.get_repo_ids_to_review() }
                            empty_repo_list_message={ "You selected no repositories to archive" }
                            { on_checkbox_change } />

            <p class="text-xl text-red-700">{
                "Clicking the 'Archive selected repositories' button will send archive requests
                 to GitHub for each of the selected repositories. This *cannot* be undone
                 here in ice-repos, and un-archiving in the GitHub web interface is possible
                 but tedious for large number of repositories. Use with caution."
            }</p>

            <div class="form-control mt-6">
                <button type="submit" class="btn btn-primary" {onclick}>{ "Archive selected repositories" }</button>
            </div>
        </div>
    }
}