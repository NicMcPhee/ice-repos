#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;
use yew_router::prelude::*;

use yewdux::prelude::*;

use ice_repos::{components::{
    welcome::Welcome,
    about::About,
    organization_entry::OrganizationEntry,
    repository_paginator::RepositoryPaginator,
}, repository::Organization};

// ===================================================================================
// for {username}.github.io/{repo_name}
// replace 'yew-template-for-github.io' to your repo name

#[derive(Clone, Routable, PartialEq)]
enum RootRoute {
    #[at("/ice-repos/")]
    Home,
    #[at("/ice-repos/:s")]
    Route,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    // #[at("/ice-repos/submit")]
    // Submit,
    #[at("/ice-repos/about")]
    About,
    #[not_found]
    #[at("/ice-repos/404")]
    NotFound,
}

fn root_route(routes: &RootRoute) -> Html {
    match routes {
        RootRoute::Home => html! { <HomePage/> },
        RootRoute::Route => html! {
            <Switch<Route> render={Switch::render(switch)} />
        },
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        // Route::Submit => html! { <Submit/> },
        Route::About => html! { <About/> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let (organization, _) = use_store::<Organization>();
    let organization = organization.name.as_ref();

    html! {
        <div class="grid grid-cols-1 divide-y flex flex-col space-y-8 m-16">
            <div class="hero min-h-fit bg-base-200">
                <div class="hero-content flex-col lg:flex-row">
                    <Welcome />
                    <OrganizationEntry/>
                </div>
            </div>

            // Where the list of repositories go
            // TODO: Maybe move this `if` into the paginator so that `HomePage` doesn't need to ever
            //   access any part of the global state. 
            if let Some(organization) = organization {
                <div>
                    <h2 class="text-2xl">{ format!("The list of repositories for the organization {}", organization) }</h2>
                    <RepositoryPaginator/>
                </div>
            }

            <div>
                <About/>
            </div>
        </div>
    }
}

// ===================================================================================

/// main root
#[function_component(App)]
fn app() -> Html {
    html! {
        // ********************************************************
        // **    basename is not supported on yew 0.19.0 yet.    **
        // <BrowserRouter basename="/ice-repos/">
        //     <Switch<Route> render={Switch::render(switch)} />
        // </BrowserRouter>
        // ********************************************************
        <BrowserRouter>
            <Switch<RootRoute> render={Switch::render(root_route)} />
        </BrowserRouter>
    }
}

/// entry point
fn main() {
    yew::start_app::<App>();
}
