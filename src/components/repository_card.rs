#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;

use crate::repository::Repository;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // TODO: Having to clone the repository in `RepositoryList` is annoying and it
    // would be cool to turn this into a reference without making a mess of the
    // memory management.
    pub repository: Repository
}

#[function_component(RepositoryCard)]
pub fn repository_card(props: &Props) -> Html {
    let Props { repository } = props;
    html! {
        <div>
            if repository.archived {
                <h2 class="text-2xl text-gray-300">{ &repository.name }</h2>
            } else {
                <h2 class="text-2xl">{ &repository.name }</h2>
            }
            {
                repository.description.as_ref().map_or_else(
                    || html! { <p class="text-blue-700">{ "There was no description for this repository "}</p> },
                    |s| html! { <p class="text-green-700">{ s }</p> }
                )
            }
            <p>{ format!("Last updated on {}", repository.updated_at.format("%Y-%m-%d")) }</p>
            <p>{ format!("Last pushed to on {}", repository.pushed_at.format("%Y-%m-%d")) }</p>
        </div>
    }
}
