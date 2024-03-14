use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::{PgPool, Pool};
// use reqwest::Client;

use crate::model::*;

pub fn registrant_router() -> Router<Pool<sqlx::Postgres>> {
    let router = Router::new()
        .route("/", post(add_registrant))
        .route("/", get(get_all_registrants));

    router
}

async fn get_all_registrants(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Registrant>("SELECT id, name, phone, message, photo FROM registrant")
        .fetch_all(&pool)
        .await
    {
        Ok(registrants) => {
            let row = registrants.into_iter().map(|reg| {
                RegistrantDTO {
                    id: reg.id,
                    name: reg.name,
                    phone: reg.phone,
                    message: reg.message,
                    photo: reg.photo,
                }
            }).collect::<Vec<RegistrantDTO>>();
            Ok((StatusCode::OK, Json(row)))
        },
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
                data.photo = "s".to_owned() //field.bytes().await.unwrap().to_vec();
            }
            _ => {}
        }
    }

    match sqlx::query_as::<_, Response>(
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
