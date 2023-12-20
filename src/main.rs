#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use yew::prelude::*;
use yew_router::prelude::*;

use yewdux::prelude::*;

use ice_repos::{
    components::{
        about::About, organization_entry::OrganizationEntry,
        repository_paginator::RepositoryPaginator, review_and_submit::ReviewAndSubmit,
        welcome::Welcome,
    },
    repository::Organization,
    Route,
};

// ===================================================================================
// for {username}.github.io/{repo_name}
// replace 'yew-template-for-github.io' to your repo name

#[derive(Clone, Routable, PartialEq, Copy)]
enum RootRoute {
    #[at("/ice-repos/")]
    Home,
    #[at("/ice-repos/:s")]
    Route,
}

fn root_route(routes: RootRoute) -> Html {
    match routes {
        RootRoute::Home => html! { <HomePage/> },
        RootRoute::Route => html! {
            <Switch<Route> render={switch} />
        },
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::ReviewAndSubmit => html! { <ReviewAndSubmit/> },
        Route::About => html! { <About/> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let (organization, _) = use_store::<Organization>();
    let organization = organization.name();

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
                    <RepositoryPaginator />
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
    // let login = Callback::from(|_: MouseEvent| {
    // OAuth2Dispatcher::<Client>::new().start_login();
    // });
    // let logout = Callback::from(|_: MouseEvent| {
    // OAuth2Dispatcher::<Client>::new().logout();
    // });

    // let config = Config {
    // client_id: "c5b735f256dadf835133".into(),
    // auth_url: "https://github.com/login/oauth/authorize".into(),
    // token_url: "http://0.0.0.0:8787/finalize_login".into(),
    // };

    return html! {
        <>
        // <p> <button onclick={logout}>{ "Logout" }</button> </p>
        // <h1>{"Authenticated!"}</h1>
        <BrowserRouter>
            <Switch<RootRoute> render={root_route}/>
        </BrowserRouter>
        </>
    };
    // return html! {
    // <OAuth2 {config}>
    // <Failure><FailureMessage/></Failure>
    // <Authenticated>
    // <p> <button onclick={logout}>{ "Logout" }</button> </p>
    // <h1>{"Authenticated!"}</h1>
    // <BrowserRouter>
    // <Switch<RootRoute> render={Switch::render(root_route)}/>
    // </BrowserRouter>
    // </Authenticated>
    // <NotAuthenticated>
    // <p>
    // { "You need to log in" }
    // </p>
    // <p>
    // <button onclick={login.clone()}>{ "Login" }</button>
    // </p>
    // </NotAuthenticated>
    // </OAuth2>
    // };

    // html! {
    //     // ********************************************************
    //     // **    basename is not supported on yew 0.19.0 yet.    **
    //     // <BrowserRouter basename="/ice-repos/">
    //     //     <Switch<Route> render={Switch::render(switch)} />
    //     // </BrowserRouter>
    //     // ********************************************************
    //     <BrowserRouter>
    //         <Switch<RootRoute> render={Switch::render(root_route)} />
    //     </BrowserRouter>
    // }
}

/// entry point
fn main() {
    yew::Renderer::<App>::new().render();
}
