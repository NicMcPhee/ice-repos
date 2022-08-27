use yew::{function_component, html};

#[function_component(Welcome)]
pub fn welcome() -> Html {
    html! {
        <div class="text-center lg:text-left">
            <h1 class="text-5xl font-bold">{ "Welcome to" }</h1>
            <h1 class="text-5xl font-bold">{ "ice-repos!" }</h1>
            <p class="py-6">{ "A tool for archiving groups of GitHub repos" }</p>
        </div>
    }
}