use web_sys::MouseEvent;
use yew::{function_component, html, Callback, Html};
use yewdux::use_store_value;

use crate::components::repository_card::ToggleState;
use crate::components::repository_list::RepositoryList;
use crate::organization::{ArchiveState, Organization, RepoFilter};

/// Review selected repositories to archive and
/// submit archive requests.
#[function_component(ReviewAndSubmit)]
pub fn review_and_submit() -> Html {
    let org = use_store_value::<Organization>();
    let onclick: Callback<MouseEvent> = Callback::noop();
    let filter = RepoFilter::review_and_submit();
    let toggle_state = ToggleState {
        on: ArchiveState::Archive,
        off: ArchiveState::KeptInReview,
    };
    let range = 0..org.repositories.len();

    // TODO: We need some kind of shared header that comes across to pages like this.
    html! {
        <div>
            <RepositoryList {range} { filter } { toggle_state }
                            empty_repo_list_message={ "You selected no repositories to archive" }/>

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
