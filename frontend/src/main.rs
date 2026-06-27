use gloo_net::http::Request;
use serde::Deserialize;
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

#[derive(Clone, Debug, PartialEq)]
enum TaskLoadStatus {
    NotLoaded,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TaskStatus {
    Pending,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Task {
    id: u32,
    title: String,
    description: String,
    status: TaskStatus,
    priority: TaskPriority,
    due_date: Option<String>,
    created_at: String,
    updated_at: String,
}

#[function_component(App)]
fn app() -> Html {
    let api_status = use_state(|| ApiStatus::NotChecked);
    let task_load_status = use_state(|| TaskLoadStatus::NotLoaded);
    let tasks = use_state(Vec::<Task>::new);

    let check_api = {
        let api_status = api_status.clone();

        Callback::from(move |_| {
            let api_status = api_status.clone();

            spawn_local(async move {
                api_status.set(ApiStatus::Loading);

                let response = Request::get(&format!("{API_BASE_URL}/health")).send().await;

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

    let load_tasks = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();

        Callback::from(move |_| {
            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();

            spawn_local(async move {
                task_load_status.set(TaskLoadStatus::Loading);

                let response = Request::get(&format!("{API_BASE_URL}/tasks"))
                    .header("x-api-key", API_KEY)
                    .send()
                    .await;

                match response {
                    Ok(res) if res.ok() => match res.json::<Vec<Task>>().await {
                        Ok(task_list) => {
                            tasks.set(task_list);
                            task_load_status.set(TaskLoadStatus::Loaded);
                        }
                        Err(error) => {
                            task_load_status.set(TaskLoadStatus::Error(format!(
                                "Could not read task JSON: {error}"
                            )));
                        }
                    },
                    Ok(res) => {
                        task_load_status.set(TaskLoadStatus::Error(format!(
                            "API returned status {}",
                            res.status()
                        )));
                    }
                    Err(error) => {
                        task_load_status.set(TaskLoadStatus::Error(format!(
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

                <div class="button-row">
                    <button onclick={check_api}>
                        { "Check Backend API" }
                    </button>

                    <button onclick={load_tasks}>
                        { "Load Tasks" }
                    </button>
                </div>

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
                <h2>{ "Tasks" }</h2>

                {
                    match &*task_load_status {
                        TaskLoadStatus::NotLoaded => html! {
                            <p>{ "Click Load Tasks to fetch tasks from the backend." }</p>
                        },
                        TaskLoadStatus::Loading => html! {
                            <p>{ "Loading tasks..." }</p>
                        },
                        TaskLoadStatus::Error(message) => html! {
                            <p class="error">{ message }</p>
                        },
                        TaskLoadStatus::Loaded => {
                            if tasks.is_empty() {
                                html! { <p>{ "No tasks found." }</p> }
                            } else {
                                html! {
                                    <div class="task-list">
                                        { for tasks.iter().map(|task| html! {
                                            <TaskCard task={task.clone()} />
                                        }) }
                                    </div>
                                }
                            }
                        }
                    }
                }
            </section>

            <section class="card">
                <h2>{ "API Configuration" }</h2>
                <p>{ format!("Backend URL: {API_BASE_URL}") }</p>
                <p>{ format!("API key used for task requests: {API_KEY}") }</p>
            </section>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TaskCardProps {
    task: Task,
}

#[function_component(TaskCard)]
fn task_card(props: &TaskCardProps) -> Html {
    let task = &props.task;

    html! {
        <article class="task-card">
            <div class="task-card-header">
                <h3>{ format!("#{} - {}", task.id, task.title) }</h3>
                <span class={status_class(&task.status)}>
                    { status_text(&task.status) }
                </span>
            </div>

            <p>{ &task.description }</p>

            <div class="task-meta">
                <span class={priority_class(&task.priority)}>
                    { format!("Priority: {}", priority_text(&task.priority)) }
                </span>

                <span class="badge">
                    {
                        match &task.due_date {
                            Some(date) => format!("Due: {date}"),
                            None => "No due date".to_string(),
                        }
                    }
                </span>
            </div>

            <div class="task-dates">
                <small>{ format!("Created: {}", task.created_at) }</small>
                <small>{ format!("Updated: {}", task.updated_at) }</small>
            </div>
        </article>
    }
}

fn status_text(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "Pending",
        TaskStatus::Completed => "Completed",
    }
}

fn priority_text(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "Low",
        TaskPriority::Medium => "Medium",
        TaskPriority::High => "High",
    }
}

fn status_class(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "badge status-pending",
        TaskStatus::Completed => "badge status-completed",
    }
}

fn priority_class(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "badge priority-low",
        TaskPriority::Medium => "badge priority-medium",
        TaskPriority::High => "badge priority-high",
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
