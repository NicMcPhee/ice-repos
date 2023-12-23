use yew::{function_component, html, Html};

#[function_component(About)]
pub fn about() -> Html {
    html! {
        <div class="mt-4">
            <p>{ "Explain the basic idea of the app here" }</p>
        </div>
    }
}
