use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{FromRow, PgPool};

async fn retrieve(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(todo) => Ok((StatusCode::OK, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add(
    State(state): State<MyState>,
    Json(data): Json<TodoNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("INSERT INTO todos (note) VALUES ($1) RETURNING id, note")
        .bind(&data.note)
        .fetch_one(&state.pool)
        .await
    {
        Ok(todo) => Ok((StatusCode::CREATED, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn get_all_registrants(
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Registrant>("SELECT id, name, phone, message FROM registrant")
        .fetch_all(&state.pool)
        .await
    {
        Ok(registrants) => Ok((StatusCode::OK, Json(registrants))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add_registrant(
    State(state): State<MyState>,
    Form(data): Form<RegistrantFormData>, // Changed to accept Form data
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Registrant>(
        "INSERT INTO registrant (name, phone, message, photo) VALUES ($1, $2, $3, $4) RETURNING id, name, phone, message",
    )
    .bind(&data.name)
    .bind(&data.phone)
    .bind(&data.message)
    .bind(&data.photo)
    .fetch_one(&state.pool)
    .await
    {
        Ok(registrant) => Ok((StatusCode::CREATED, Json(registrant))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };
    let router = Router::new()
        .route("/todos", post(add))
        .route("/todos/:id", get(retrieve))
        .route("/registrants", post(add_registrant)) // New route for adding registrants
        .route("/registrants", get(get_all_registrants)) // New route for getting all registrants
        .with_state(state);

    Ok(router.into())
}

#[derive(Deserialize)] // Changed to match form data
struct RegistrantFormData {
    pub name: String,
    pub phone: String,
    pub message: String,
    pub photo: Vec<u8>, // Assuming the photo is provided as a byte array
}

#[derive(Serialize, FromRow)]
struct Registrant {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub message: String,
}

#[derive(Deserialize)]
struct TodoNew {
    pub note: String,
}

#[derive(Serialize, FromRow)]
struct Todo {
    pub id: i32,
    pub note: String,
}
