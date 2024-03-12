use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{FromRow, PgPool};

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
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut data = RegistrantFormData::default();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "name" => data.name = field.text().await.unwrap(),
            "phone" => data.phone = field.text().await.unwrap(),
            "message" => data.message = field.text().await.unwrap(),
            "photo" => {
                data.photo = field.bytes().await.unwrap().to_vec();
            }
            _ => {}
        }
    }

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
        Ok(row) => Ok((StatusCode::CREATED, Json(row))),
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
        .route("/registrants", post(add_registrant))
        .route("/registrants", get(get_all_registrants))
        .with_state(state);

    Ok(router.into())
}

#[derive(Default, Deserialize)]
struct RegistrantFormData {
    pub name: String,
    pub phone: String,
    pub message: String,
    pub photo: Vec<u8>,
}

#[derive(Serialize, FromRow)]
struct Registrant {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub message: String,
}
