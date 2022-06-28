use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;

use yew::callback;
use yew::prelude::*;
use yew_router::prelude::*;

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
    #[at("/ice-repos/about")]
    About,
    #[not_found]
    #[at("/ice-repos/404")]
    NotFound,
}

fn root_route(routes: &RootRoute) -> Html {
    match routes {
        RootRoute::Home => home_page(),
        RootRoute::Route => html! {
            <Switch<Route> render={Switch::render(switch)} />
        },
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct TextInputProps {
    pub value: String,
    pub on_change: Callback<String>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    web_sys::console::log_1(&target.value().into());
    target.value()
}

// * The callbacks aren't right – we pass back the text input contents on *every* change when it should be on input.
//   * We made progress (?) on this, but it's still logging every keystroke instead of just
//     when the button is pushed.
// * Change the state when the text area loses focus instead of requiring a click on the submit button.
// * Convert the state back to &str to avoid all the copying.

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let field_contents = use_state(|| String::from(""));

    let TextInputProps { value, on_change } = props.clone();

    let oninput = {
        let field_contents = field_contents.clone();
        Callback::from(move |input_event: InputEvent| {
            field_contents.set(get_value_from_input_event(input_event))
        })
    };

    let onclick: Callback<MouseEvent> = {
        let field_contents = field_contents.clone();
        Callback::from(move |mouse_event: MouseEvent| {
            on_change.emit((*field_contents).clone());
        })
    };

    html! {
        <div>
            <input type="text" {oninput} value={ (*field_contents).clone() } size=40 placeholder="Enter an organization name here" />
            <button {onclick}>{ "Submit" }</button>
        </div>
    }
}

fn home_page() -> Html {
    let on_change: Callback<String> = Callback::from(|string| { 
        web_sys::console::log_1(&format!("We got <{}> from the text input!", string).into()) 
    });
    html! {
        <div>
            <div>
                <p class="text-4xl">{ "Welcome to Ice Repos" }</p> 
                <p class="text-2xl">{ "A tool for archiving groups of GitHub repos" }</p> 
            </div>

            <div>
                <p>{ "Put form stuff here" }</p>

                <div>
                    <p>{ "Enter either an organization or a GitHub Classroom"}</p>
                    <TextInput {on_change} value=""/>
                </div>
            </div>

            <div>
                <p>{ "Put the about stuff here" }</p>
            </div>
        </div>
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::About => html! { <p>{ "About" }</p> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

// ===================================================================================
// for {username}.github.io

// #[derive(Clone, Routable, PartialEq)]
//  enum RootRoute {
//      #[at("/")]
//      Home,
//      #[at("/about")]
//      About,
//      #[not_found]
//      #[at("/404")]
//      NotFound,
//  }

//  fn root_route(routes: &Route) -> Html {
//      match routes {
//          RootRoute::Home => html! { <p class="text-4xl">{ "Yew Template" }</p> },
//          RootRoute::About => html! { <p>{ "About" }</p> },
//          RootRoute::NotFound => html! { <p>{ "Not Found" }</p> },
//      }
//  }

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
