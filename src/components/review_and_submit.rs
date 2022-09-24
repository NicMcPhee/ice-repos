use yew::{function_component, html, Callback};
use yewdux::prelude::use_store;

use crate::repository::{ArchiveStateMap, DesiredArchiveState, ArchiveState};
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
                archive_state_map.update_desired_state(id, ArchiveState::from_review_state(desired_archive_state));
            });
        })
    };

    // TODO: We need some kind of shared header that comes across to pages like this.

    // TODO: We need a "Submit" button that will actually spin up all the archiving
    //   requests.

    // TODO: This is (almost) exactly the same as the code in `RepositoryList`;
    //   there perhaps should be a new component that just displays a list of
    //   repository cards that can be shared between this and `RepositoryList`.
    archive_state_map.get_repos_to_review().map(|repository| {
        html! {
            <RepositoryCard repository={ repository.clone() }
                            desired_archive_state={ archive_state_map.get_desired_state(repository.id) }
                            on_checkbox_change={ on_checkbox_change.clone() } 
                            />
        }
    }).collect()
}