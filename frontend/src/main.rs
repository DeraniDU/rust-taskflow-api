use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, SubmitEvent};
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

#[derive(Clone, Debug, PartialEq)]
enum CreateTaskStatus {
    Idle,
    Creating,
    Success(String),
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
enum TaskActionStatus {
    Idle,
    Processing,
    Success(String),
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

#[derive(Clone, Debug, Serialize)]
struct CreateTaskPayload {
    title: String,
    description: String,
    priority: String,
    due_date: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct UpdateTaskPayload {
    title: String,
    description: String,
    status: String,
    priority: String,
    due_date: Option<String>,
}

async fn fetch_tasks_from_api() -> Result<Vec<Task>, String> {
    let response = Request::get(&format!("{API_BASE_URL}/tasks"))
        .header("x-api-key", API_KEY)
        .send()
        .await
        .map_err(|error| format!("Could not connect to backend: {error}"))?;

    if !response.ok() {
        return Err(format!("API returned status {}", response.status()));
    }

    response
        .json::<Vec<Task>>()
        .await
        .map_err(|error| format!("Could not read task JSON: {error}"))
}

async fn create_task_in_api(payload: CreateTaskPayload) -> Result<Task, String> {
    let request = Request::post(&format!("{API_BASE_URL}/tasks"))
        .header("Content-Type", "application/json")
        .header("x-api-key", API_KEY)
        .json(&payload)
        .map_err(|error| format!("Could not prepare request body: {error}"))?;

    let response = request
        .send()
        .await
        .map_err(|error| format!("Could not connect to backend: {error}"))?;

    if !response.ok() {
        return Err(format!("API returned status {}", response.status()));
    }

    response
        .json::<Task>()
        .await
        .map_err(|error| format!("Could not read created task JSON: {error}"))
}

async fn update_task_in_api(task_id: u32, payload: UpdateTaskPayload) -> Result<Task, String> {
    let request = Request::put(&format!("{API_BASE_URL}/tasks/{task_id}"))
        .header("Content-Type", "application/json")
        .header("x-api-key", API_KEY)
        .json(&payload)
        .map_err(|error| format!("Could not prepare update request body: {error}"))?;

    let response = request
        .send()
        .await
        .map_err(|error| format!("Could not connect to backend: {error}"))?;

    if !response.ok() {
        return Err(format!("API returned status {}", response.status()));
    }

    response
        .json::<Task>()
        .await
        .map_err(|error| format!("Could not read updated task JSON: {error}"))
}

async fn delete_task_in_api(task_id: u32) -> Result<(), String> {
    let response = Request::delete(&format!("{API_BASE_URL}/tasks/{task_id}"))
        .header("x-api-key", API_KEY)
        .send()
        .await
        .map_err(|error| format!("Could not connect to backend: {error}"))?;

    if !response.ok() {
        return Err(format!("API returned status {}", response.status()));
    }

    Ok(())
}

#[function_component(App)]
fn app() -> Html {
    let api_status = use_state(|| ApiStatus::NotChecked);
    let task_load_status = use_state(|| TaskLoadStatus::NotLoaded);
    let create_task_status = use_state(|| CreateTaskStatus::Idle);
    let task_action_status = use_state(|| TaskActionStatus::Idle);
    let tasks = use_state(Vec::<Task>::new);

    let new_title = use_state(String::new);
    let new_description = use_state(String::new);
    let new_priority = use_state(|| "medium".to_string());
    let new_due_date = use_state(String::new);

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

                match fetch_tasks_from_api().await {
                    Ok(task_list) => {
                        tasks.set(task_list);
                        task_load_status.set(TaskLoadStatus::Loaded);
                    }
                    Err(message) => {
                        task_load_status.set(TaskLoadStatus::Error(message));
                    }
                }
            });
        })
    };

    let on_title_input = {
        let new_title = new_title.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            new_title.set(input.value());
        })
    };

    let on_description_input = {
        let new_description = new_description.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target_unchecked_into();
            new_description.set(input.value());
        })
    };

    let on_priority_change = {
        let new_priority = new_priority.clone();

        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            new_priority.set(select.value());
        })
    };

    let on_due_date_input = {
        let new_due_date = new_due_date.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            new_due_date.set(input.value());
        })
    };

    let on_create_task = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let create_task_status = create_task_status.clone();

        let new_title = new_title.clone();
        let new_description = new_description.clone();
        let new_priority = new_priority.clone();
        let new_due_date = new_due_date.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let title = (*new_title).trim().to_string();
            let description = (*new_description).trim().to_string();
            let priority = (*new_priority).clone();
            let due_date_text = (*new_due_date).trim().to_string();

            if title.is_empty() {
                create_task_status.set(CreateTaskStatus::Error(
                    "Task title is required.".to_string(),
                ));
                return;
            }

            let payload = CreateTaskPayload {
                title,
                description,
                priority,
                due_date: if due_date_text.is_empty() {
                    None
                } else {
                    Some(due_date_text)
                },
            };

            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();
            let create_task_status = create_task_status.clone();

            let new_title = new_title.clone();
            let new_description = new_description.clone();
            let new_priority = new_priority.clone();
            let new_due_date = new_due_date.clone();

            spawn_local(async move {
                create_task_status.set(CreateTaskStatus::Creating);

                match create_task_in_api(payload).await {
                    Ok(created_task) => {
                        create_task_status.set(CreateTaskStatus::Success(format!(
                            "Task #{} created successfully.",
                            created_task.id
                        )));

                        new_title.set(String::new());
                        new_description.set(String::new());
                        new_priority.set("medium".to_string());
                        new_due_date.set(String::new());

                        task_load_status.set(TaskLoadStatus::Loading);

                        match fetch_tasks_from_api().await {
                            Ok(task_list) => {
                                tasks.set(task_list);
                                task_load_status.set(TaskLoadStatus::Loaded);
                            }
                            Err(message) => {
                                task_load_status.set(TaskLoadStatus::Error(message));
                            }
                        }
                    }
                    Err(message) => {
                        create_task_status.set(CreateTaskStatus::Error(message));
                    }
                }
            });
        })
    };

    let on_mark_completed = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let task_action_status = task_action_status.clone();

        Callback::from(move |task: Task| {
            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();
            let task_action_status = task_action_status.clone();

            spawn_local(async move {
                task_action_status.set(TaskActionStatus::Processing);

                let payload = UpdateTaskPayload {
                    title: task.title.clone(),
                    description: task.description.clone(),
                    status: "completed".to_string(),
                    priority: priority_api_value(&task.priority).to_string(),
                    due_date: task.due_date.clone(),
                };

                match update_task_in_api(task.id, payload).await {
                    Ok(updated_task) => {
                        task_action_status.set(TaskActionStatus::Success(format!(
                            "Task #{} marked as completed.",
                            updated_task.id
                        )));

                        task_load_status.set(TaskLoadStatus::Loading);

                        match fetch_tasks_from_api().await {
                            Ok(task_list) => {
                                tasks.set(task_list);
                                task_load_status.set(TaskLoadStatus::Loaded);
                            }
                            Err(message) => {
                                task_load_status.set(TaskLoadStatus::Error(message));
                            }
                        }
                    }
                    Err(message) => {
                        task_action_status.set(TaskActionStatus::Error(message));
                    }
                }
            });
        })
    };

    let on_delete_task = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let task_action_status = task_action_status.clone();

        Callback::from(move |task_id: u32| {
            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();
            let task_action_status = task_action_status.clone();

            spawn_local(async move {
                task_action_status.set(TaskActionStatus::Processing);

                match delete_task_in_api(task_id).await {
                    Ok(()) => {
                        task_action_status.set(TaskActionStatus::Success(format!(
                            "Task #{task_id} deleted successfully."
                        )));

                        task_load_status.set(TaskLoadStatus::Loading);

                        match fetch_tasks_from_api().await {
                            Ok(task_list) => {
                                tasks.set(task_list);
                                task_load_status.set(TaskLoadStatus::Loaded);
                            }
                            Err(message) => {
                                task_load_status.set(TaskLoadStatus::Error(message));
                            }
                        }
                    }
                    Err(message) => {
                        task_action_status.set(TaskActionStatus::Error(message));
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
                    <button onclick={check_api}>{ "Check Backend API" }</button>
                    <button onclick={load_tasks}>{ "Load Tasks" }</button>
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
                <h2>{ "Create New Task" }</h2>

                <form class="task-form" onsubmit={on_create_task}>
                    <label>
                        { "Title" }
                        <input
                            type="text"
                            placeholder="Enter task title"
                            value={(*new_title).clone()}
                            oninput={on_title_input}
                        />
                    </label>

                    <label>
                        { "Description" }
                        <textarea
                            placeholder="Enter task description"
                            value={(*new_description).clone()}
                            oninput={on_description_input}
                        />
                    </label>

                    <div class="form-row">
                        <label>
                            { "Priority" }
                            <select value={(*new_priority).clone()} onchange={on_priority_change}>
                                <option value="low">{ "Low" }</option>
                                <option value="medium">{ "Medium" }</option>
                                <option value="high">{ "High" }</option>
                            </select>
                        </label>

                        <label>
                            { "Due Date" }
                            <input
                                type="date"
                                value={(*new_due_date).clone()}
                                oninput={on_due_date_input}
                            />
                        </label>
                    </div>

                    <button type="submit">
                        {
                            match &*create_task_status {
                                CreateTaskStatus::Creating => "Creating...",
                                _ => "Create Task",
                            }
                        }
                    </button>
                </form>

                <p>
                    {
                        match &*create_task_status {
                            CreateTaskStatus::Idle => html! {},
                            CreateTaskStatus::Creating => html! { <span>{ "Creating task..." }</span> },
                            CreateTaskStatus::Success(message) => html! { <span class="success">{ message }</span> },
                            CreateTaskStatus::Error(message) => html! { <span class="error">{ message }</span> },
                        }
                    }
                </p>
            </section>

            <section class="card">
                <h2>{ "Task Actions" }</h2>

                <p>
                    {
                        match &*task_action_status {
                            TaskActionStatus::Idle => html! { <span>{ "No task action yet." }</span> },
                            TaskActionStatus::Processing => html! { <span>{ "Processing task action..." }</span> },
                            TaskActionStatus::Success(message) => html! { <span class="success">{ message }</span> },
                            TaskActionStatus::Error(message) => html! { <span class="error">{ message }</span> },
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
                                            <TaskCard
                                                task={task.clone()}
                                                on_mark_completed={on_mark_completed.clone()}
                                                on_delete_task={on_delete_task.clone()}
                                            />
                                        }) }
                                    </div>
                                }
                            }
                        }
                    }
                }
            </section>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TaskCardProps {
    task: Task,
    on_mark_completed: Callback<Task>,
    on_delete_task: Callback<u32>,
}

#[function_component(TaskCard)]
fn task_card(props: &TaskCardProps) -> Html {
    let task = &props.task;

    let mark_completed = {
        let task = task.clone();
        let on_mark_completed = props.on_mark_completed.clone();

        Callback::from(move |_| {
            on_mark_completed.emit(task.clone());
        })
    };

    let delete_task = {
        let task_id = task.id;
        let on_delete_task = props.on_delete_task.clone();

        Callback::from(move |_| {
            on_delete_task.emit(task_id);
        })
    };

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

            <div class="task-actions">
                {
                    if task.status == TaskStatus::Pending {
                        html! {
                            <button class="complete-button" onclick={mark_completed}>
                                { "Mark Completed" }
                            </button>
                        }
                    } else {
                        html! {
                            <button class="complete-button" disabled=true>
                                { "Completed" }
                            </button>
                        }
                    }
                }

                <button class="delete-button" onclick={delete_task}>
                    { "Delete" }
                </button>
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

fn priority_api_value(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "low",
        TaskPriority::Medium => "medium",
        TaskPriority::High => "high",
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
