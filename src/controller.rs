use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::{PgPool, Pool};

use crate::model::{Registrant, RegistrantFormData};

pub fn registrant_router() -> Router<Pool<sqlx::Postgres>> {
    let router = Router::new()
        .route("/", post(add_registrant))
        .route("/", get(get_all_registrants));

    router
}

async fn get_all_registrants(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Registrant>("SELECT id, name, phone, message FROM registrant")
        .fetch_all(&pool)
        .await
    {
        Ok(registrants) => Ok((StatusCode::OK, Json(registrants))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add_registrant(
    State(pool): State<PgPool>,
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
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok((StatusCode::CREATED, Json(row))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}
