#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![deny(bindings_with_variant_name)]

use yew_router::Routable;

pub mod services;
pub mod components;
pub mod repository;

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/ice-repos/review-and-submit")]
    ReviewAndSubmit,
    #[at("/ice-repos/about")]
    About,
    #[not_found]
    #[at("/ice-repos/404")]
    NotFound,
}
