use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
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

#[derive(Clone, Debug, PartialEq)]
struct TaskFilters {
    status: String,
    priority: String,
    due_date: String,
}

#[derive(Clone, Debug, Serialize)]
struct CreateTaskPayload {
    title: String,
    description: String,
    priority: String,
    due_date: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct UpdateTaskPayload {
    title: String,
    description: String,
    status: String,
    priority: String,
    due_date: Option<String>,
}

fn default_filters() -> TaskFilters {
    TaskFilters {
        status: "all".to_string(),
        priority: "all".to_string(),
        due_date: String::new(),
    }
}

fn build_tasks_url(filters: &TaskFilters) -> String {
    let mut query_params = Vec::new();

    if filters.status != "all" {
        query_params.push(format!("status={}", filters.status));
    }

    if filters.priority != "all" {
        query_params.push(format!("priority={}", filters.priority));
    }

    if !filters.due_date.trim().is_empty() {
        query_params.push(format!("due_date={}", filters.due_date.trim()));
    }

    if query_params.is_empty() {
        format!("{API_BASE_URL}/tasks")
    } else {
        format!("{API_BASE_URL}/tasks?{}", query_params.join("&"))
    }
}

async fn fetch_tasks_from_api(filters: TaskFilters) -> Result<Vec<Task>, String> {
    let url = build_tasks_url(&filters);

    let response = Request::get(&url)
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

fn load_tasks_with_filters(
    filters: TaskFilters,
    tasks: UseStateHandle<Vec<Task>>,
    task_load_status: UseStateHandle<TaskLoadStatus>,
) {
    spawn_local(async move {
        task_load_status.set(TaskLoadStatus::Loading);

        match fetch_tasks_from_api(filters).await {
            Ok(task_list) => {
                tasks.set(task_list);
                task_load_status.set(TaskLoadStatus::Loaded);
            }
            Err(message) => {
                task_load_status.set(TaskLoadStatus::Error(message));
            }
        }
    });
}

#[function_component(App)]
fn app() -> Html {
    let api_status = use_state(|| ApiStatus::NotChecked);
    let task_load_status = use_state(|| TaskLoadStatus::NotLoaded);
    let create_task_status = use_state(|| CreateTaskStatus::Idle);
    let task_action_status = use_state(|| TaskActionStatus::Idle);
    let tasks = use_state(Vec::<Task>::new);

    let filters = use_state(default_filters);

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
        let filters = filters.clone();

        Callback::from(move |_| {
            load_tasks_with_filters((*filters).clone(), tasks.clone(), task_load_status.clone());
        })
    };

    let on_status_filter_change = {
        let filters = filters.clone();

        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            let mut updated_filters = (*filters).clone();
            updated_filters.status = select.value();
            filters.set(updated_filters);
        })
    };

    let on_priority_filter_change = {
        let filters = filters.clone();

        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            let mut updated_filters = (*filters).clone();
            updated_filters.priority = select.value();
            filters.set(updated_filters);
        })
    };

    let on_due_date_filter_input = {
        let filters = filters.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            let mut updated_filters = (*filters).clone();
            updated_filters.due_date = input.value();
            filters.set(updated_filters);
        })
    };

    let apply_filters = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let filters = filters.clone();

        Callback::from(move |_| {
            load_tasks_with_filters((*filters).clone(), tasks.clone(), task_load_status.clone());
        })
    };

    let clear_filters = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let filters = filters.clone();

        Callback::from(move |_| {
            let clean_filters = default_filters();
            filters.set(clean_filters.clone());
            load_tasks_with_filters(clean_filters, tasks.clone(), task_load_status.clone());
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
        let filters = filters.clone();

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
            let filters = filters.clone();

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

                        load_tasks_with_filters(
                            (*filters).clone(),
                            tasks.clone(),
                            task_load_status.clone(),
                        );
                    }
                    Err(message) => {
                        create_task_status.set(CreateTaskStatus::Error(message));
                    }
                }
            });
        })
    };

    let on_update_task = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let task_action_status = task_action_status.clone();
        let filters = filters.clone();

        Callback::from(move |(task_id, payload): (u32, UpdateTaskPayload)| {
            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();
            let task_action_status = task_action_status.clone();
            let filters = filters.clone();

            spawn_local(async move {
                task_action_status.set(TaskActionStatus::Processing);

                match update_task_in_api(task_id, payload).await {
                    Ok(updated_task) => {
                        task_action_status.set(TaskActionStatus::Success(format!(
                            "Task #{} updated successfully.",
                            updated_task.id
                        )));

                        load_tasks_with_filters(
                            (*filters).clone(),
                            tasks.clone(),
                            task_load_status.clone(),
                        );
                    }
                    Err(message) => {
                        task_action_status.set(TaskActionStatus::Error(message));
                    }
                }
            });
        })
    };

    let on_mark_completed = {
        let on_update_task = on_update_task.clone();

        Callback::from(move |task: Task| {
            let payload = UpdateTaskPayload {
                title: task.title.clone(),
                description: task.description.clone(),
                status: "completed".to_string(),
                priority: priority_api_value(&task.priority).to_string(),
                due_date: task.due_date.clone(),
            };

            on_update_task.emit((task.id, payload));
        })
    };

    let on_delete_task = {
        let tasks = tasks.clone();
        let task_load_status = task_load_status.clone();
        let task_action_status = task_action_status.clone();
        let filters = filters.clone();

        Callback::from(move |task_id: u32| {
            let tasks = tasks.clone();
            let task_load_status = task_load_status.clone();
            let task_action_status = task_action_status.clone();
            let filters = filters.clone();

            spawn_local(async move {
                task_action_status.set(TaskActionStatus::Processing);

                match delete_task_in_api(task_id).await {
                    Ok(()) => {
                        task_action_status.set(TaskActionStatus::Success(format!(
                            "Task #{task_id} deleted successfully."
                        )));

                        load_tasks_with_filters(
                            (*filters).clone(),
                            tasks.clone(),
                            task_load_status.clone(),
                        );
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
                <h2>{ "Task Filters" }</h2>

                <div class="filter-form">
                    <label>
                        { "Status" }
                        <select value={filters.status.clone()} onchange={on_status_filter_change}>
                            <option value="all">{ "All" }</option>
                            <option value="pending">{ "Pending" }</option>
                            <option value="completed">{ "Completed" }</option>
                        </select>
                    </label>

                    <label>
                        { "Priority" }
                        <select value={filters.priority.clone()} onchange={on_priority_filter_change}>
                            <option value="all">{ "All" }</option>
                            <option value="low">{ "Low" }</option>
                            <option value="medium">{ "Medium" }</option>
                            <option value="high">{ "High" }</option>
                        </select>
                    </label>

                    <label>
                        { "Due Date" }
                        <input
                            type="date"
                            value={filters.due_date.clone()}
                            oninput={on_due_date_filter_input}
                        />
                    </label>
                </div>

                <div class="button-row">
                    <button onclick={apply_filters}>{ "Apply Filters" }</button>
                    <button class="secondary-button" onclick={clear_filters}>{ "Clear Filters" }</button>
                </div>

                <p class="filter-preview">
                    { format!("Current API query: {}", build_tasks_url(&filters)) }
                </p>
            </section>

            <section class="card">
                <h2>{ "Task Summary" }</h2>
                <TaskSummary tasks={(*tasks).clone()} />
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
                                html! { <p>{ "No tasks found for selected filters." }</p> }
                            } else {
                                html! {
                                    <div class="task-list">
                                        { for tasks.iter().map(|task| html! {
                                            <TaskCard
                                                task={task.clone()}
                                                on_mark_completed={on_mark_completed.clone()}
                                                on_update_task={on_update_task.clone()}
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
struct TaskSummaryProps {
    tasks: Vec<Task>,
}

#[function_component(TaskSummary)]
fn task_summary(props: &TaskSummaryProps) -> Html {
    let total_tasks = props.tasks.len();

    let pending_tasks = props
        .tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Pending)
        .count();

    let completed_tasks = props
        .tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Completed)
        .count();

    let high_priority_tasks = props
        .tasks
        .iter()
        .filter(|task| task.priority == TaskPriority::High)
        .count();

    html! {
        <div class="summary-grid">
            <div class="summary-card">
                <span class="summary-number">{ total_tasks }</span>
                <span class="summary-label">{ "Total Tasks" }</span>
            </div>

            <div class="summary-card">
                <span class="summary-number">{ pending_tasks }</span>
                <span class="summary-label">{ "Pending" }</span>
            </div>

            <div class="summary-card">
                <span class="summary-number">{ completed_tasks }</span>
                <span class="summary-label">{ "Completed" }</span>
            </div>

            <div class="summary-card">
                <span class="summary-number">{ high_priority_tasks }</span>
                <span class="summary-label">{ "High Priority" }</span>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TaskCardProps {
    task: Task,
    on_mark_completed: Callback<Task>,
    on_update_task: Callback<(u32, UpdateTaskPayload)>,
    on_delete_task: Callback<u32>,
}

#[function_component(TaskCard)]
fn task_card(props: &TaskCardProps) -> Html {
    let task = &props.task;

    let is_editing = use_state(|| false);
    let edit_title = use_state(|| task.title.clone());
    let edit_description = use_state(|| task.description.clone());
    let edit_status = use_state(|| status_api_value(&task.status).to_string());
    let edit_priority = use_state(|| priority_api_value(&task.priority).to_string());
    let edit_due_date = use_state(|| task.due_date.clone().unwrap_or_default());

    let start_edit = {
        let is_editing = is_editing.clone();
        let edit_title = edit_title.clone();
        let edit_description = edit_description.clone();
        let edit_status = edit_status.clone();
        let edit_priority = edit_priority.clone();
        let edit_due_date = edit_due_date.clone();
        let task = task.clone();

        Callback::from(move |_| {
            edit_title.set(task.title.clone());
            edit_description.set(task.description.clone());
            edit_status.set(status_api_value(&task.status).to_string());
            edit_priority.set(priority_api_value(&task.priority).to_string());
            edit_due_date.set(task.due_date.clone().unwrap_or_default());
            is_editing.set(true);
        })
    };

    let cancel_edit = {
        let is_editing = is_editing.clone();

        Callback::from(move |_| {
            is_editing.set(false);
        })
    };

    let on_edit_title_input = {
        let edit_title = edit_title.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            edit_title.set(input.value());
        })
    };

    let on_edit_description_input = {
        let edit_description = edit_description.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target_unchecked_into();
            edit_description.set(input.value());
        })
    };

    let on_edit_status_change = {
        let edit_status = edit_status.clone();

        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            edit_status.set(select.value());
        })
    };

    let on_edit_priority_change = {
        let edit_priority = edit_priority.clone();

        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            edit_priority.set(select.value());
        })
    };

    let on_edit_due_date_input = {
        let edit_due_date = edit_due_date.clone();

        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            edit_due_date.set(input.value());
        })
    };

    let save_edit = {
        let task_id = task.id;
        let edit_title = edit_title.clone();
        let edit_description = edit_description.clone();
        let edit_status = edit_status.clone();
        let edit_priority = edit_priority.clone();
        let edit_due_date = edit_due_date.clone();
        let is_editing = is_editing.clone();
        let on_update_task = props.on_update_task.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let title = (*edit_title).trim().to_string();

            if title.is_empty() {
                return;
            }

            let due_date_text = (*edit_due_date).trim().to_string();

            let payload = UpdateTaskPayload {
                title,
                description: (*edit_description).trim().to_string(),
                status: (*edit_status).clone(),
                priority: (*edit_priority).clone(),
                due_date: if due_date_text.is_empty() {
                    None
                } else {
                    Some(due_date_text)
                },
            };

            on_update_task.emit((task_id, payload));
            is_editing.set(false);
        })
    };

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

    if *is_editing {
        html! {
            <article class="task-card editing-card">
                <h3>{ format!("Edit Task #{}", task.id) }</h3>

                <form class="task-form" onsubmit={save_edit}>
                    <label>
                        { "Title" }
                        <input
                            type="text"
                            value={(*edit_title).clone()}
                            oninput={on_edit_title_input}
                        />
                    </label>

                    <label>
                        { "Description" }
                        <textarea
                            value={(*edit_description).clone()}
                            oninput={on_edit_description_input}
                        />
                    </label>

                    <div class="form-row">
                        <label>
                            { "Status" }
                            <select value={(*edit_status).clone()} onchange={on_edit_status_change}>
                                <option value="pending">{ "Pending" }</option>
                                <option value="completed">{ "Completed" }</option>
                            </select>
                        </label>

                        <label>
                            { "Priority" }
                            <select value={(*edit_priority).clone()} onchange={on_edit_priority_change}>
                                <option value="low">{ "Low" }</option>
                                <option value="medium">{ "Medium" }</option>
                                <option value="high">{ "High" }</option>
                            </select>
                        </label>

                        <label>
                            { "Due Date" }
                            <input
                                type="date"
                                value={(*edit_due_date).clone()}
                                oninput={on_edit_due_date_input}
                            />
                        </label>
                    </div>

                    <div class="task-actions">
                        <button type="submit" class="save-button">{ "Save Changes" }</button>
                        <button type="button" class="secondary-button" onclick={cancel_edit}>{ "Cancel" }</button>
                    </div>
                </form>
            </article>
        }
    } else {
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
                    <button class="edit-button" onclick={start_edit}>
                        { "Edit" }
                    </button>

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

fn status_api_value(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Completed => "completed",
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
