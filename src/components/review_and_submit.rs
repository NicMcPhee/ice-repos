use yew::{function_component, html, Callback};
use yewdux::prelude::use_store;

use crate::repository::{ArchiveStateMap, DesiredArchiveState};
use crate::components::repository_card::RepositoryCard;

/// Review selected repositories to archive and
/// submit archive requests.
#[function_component(ReviewAndSubmit)]
pub fn review_and_submit() -> Html {
    let (archive_state_map, archive_state_dispatch) = use_store::<ArchiveStateMap>();

    let on_checkbox_change: Callback<DesiredArchiveState> = {
        Callback::from(move |desired_archive_state| {
            let DesiredArchiveState { id, desired_archive_state } = desired_archive_state;
            archive_state_dispatch.reduce_mut(|archive_state_map| {
                archive_state_map.update_desired_state(id, desired_archive_state);
            });
        })
    };

    // TODO: We need some kind of shared header that comes across to pages like this.

    // TODO: We need a "Submit" button that will actually spin up all the archiving
    //   requests.

    // TODO: This is (almost) exactly the same as the code in `RepositoryList`;
    //   there perhaps should be a new component that just displays a list of
    //   repository cards that can be shared between this and `RepositoryList`.

    // TODO: Currently repositories disappear from the list when unchecked, which
    //   almost certainly violates the principle of least amazement. I'll need
    //   look into that. An option suggested by @esitsu is to not allow the user
    //   to change the status here, but instead force them to go back to the
    //   RepositoryPaginator interface to change the state of a repository.
    //   If we do that it would be nice to have a "go back" button on each repo
    //   so we can go back to that repo's page, making it so the user doesn't
    //   have to page through, hunting a particular repository.
    archive_state_map.get_repos_to_archive().map(|repository| {
        html! {
            <RepositoryCard repository={ repository.clone() }
                            desired_archive_state={ archive_state_map.get_desired_state(repository.id) }
                            on_checkbox_change={ on_checkbox_change.clone() } 
                            />
        }
    }).collect()
}