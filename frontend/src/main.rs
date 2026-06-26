use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

const API_BASE_URL: &str = "http://127.0.0.1:3000";
const API_KEY: &str = "dev-secret-key";

#[derive(Clone, Debug, PartialEq)]
enum ApiStatus {
    NotChecked,
    Loading,
    Online,
    Offline(String),
}

#[function_component(App)]
fn app() -> Html {
    let api_status = use_state(|| ApiStatus::NotChecked);

    let check_api = {
        let api_status = api_status.clone();

        Callback::from(move |_| {
            let api_status = api_status.clone();

            spawn_local(async move {
                api_status.set(ApiStatus::Loading);

                let response = Request::get(&format!("{API_BASE_URL}/health"))
                    .send()
                    .await;

                match response {
                    Ok(res) if res.ok() => {
                        api_status.set(ApiStatus::Online);
                    }
                    Ok(res) => {
                        api_status.set(ApiStatus::Offline(format!(
                            "API returned status {}",
                            res.status()
                        )));
                    }
                    Err(error) => {
                        api_status.set(ApiStatus::Offline(format!(
                            "Could not connect to backend: {error}"
                        )));
                    }
                }
            });
        })
    };

    html! {
        <div class="container">
            <section class="header">
                <h1>{ "Rust TaskFlow" }</h1>
                <p>{ "Full-stack Rust task management app using Axum backend and Yew frontend." }</p>

                <button onclick={check_api}>
                    { "Check Backend API" }
                </button>

                <p>
                    {
                        match &*api_status {
                            ApiStatus::NotChecked => html! { <span>{ "API status not checked yet." }</span> },
                            ApiStatus::Loading => html! { <span>{ "Checking backend..." }</span> },
                            ApiStatus::Online => html! { <span class="success">{ "Backend API is online." }</span> },
                            ApiStatus::Offline(message) => html! { <span class="error">{ message }</span> },
                        }
                    }
                </p>
            </section>

            <section class="card">
                <h2>{ "Frontend Setup Complete" }</h2>
                <p>{ "This is the first Yew frontend page for the Rust TaskFlow API." }</p>

                <span class="badge">{ "Rust" }</span>
                <span class="badge">{ "Yew" }</span>
                <span class="badge">{ "Trunk" }</span>
                <span class="badge">{ "WebAssembly" }</span>
            </section>

            <section class="card">
                <h2>{ "Next Frontend Features" }</h2>
                <ul>
                    <li>{ "Display all tasks" }</li>
                    <li>{ "Create task form" }</li>
                    <li>{ "Update task" }</li>
                    <li>{ "Delete task" }</li>
                    <li>{ "Filter tasks" }</li>
                </ul>
            </section>

            <section class="card">
                <h2>{ "API Configuration" }</h2>
                <p>{ format!("Backend URL: {API_BASE_URL}") }</p>
                <p>{ format!("API key used for future task requests: {API_KEY}") }</p>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
