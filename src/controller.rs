use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::multipart::Form;
use serde_json;

use crate::model::*;

pub fn registrant_router() -> Router<Statex> {
    let router = Router::new()
        .route("/", post(add_registrant))
        .route("/", get(get_all_registrants));

    router
}

async fn get_all_registrants(
    State(stx): State<Statex>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Registrant>("SELECT id, name, phone, message, photo FROM registrant")
        .fetch_all(&stx.pool)
        .await
    {
        Ok(registrants) => Ok((StatusCode::OK, Json(registrants))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add_registrant(
    State(stx): State<Statex>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut data = RegistrantDTO::default();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "name" => data.name = field.text().await.unwrap(),
            "phone" => data.phone = field.text().await.unwrap(),
            "message" => data.message = field.text().await.unwrap(),
            "photo" => data.photo = field.bytes().await.unwrap().to_vec(),
            _ => {}
        }
    }

    let client = reqwest::Client::new();
    let form_data = Form::new()
        .text("title", format!("{}", data.name))
        .text(
            "description",
            format!("{} --- {}", data.phone, data.message),
        )
        .part(
            "image",
            reqwest::multipart::Part::bytes(data.photo),
        );

    let imgur_response = client
        .post("https://api.imgur.com/3/image")
        .header("Authorization", format!("Bearer {}", &stx.token))
        .multipart(form_data)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .text()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let imgur: ImgurResponse = serde_json::from_str(&imgur_response)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !imgur.success {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Upload photo failed".to_string(),
        ));
    }

    let row = sqlx::query_as::<_, Registrant>(
        "INSERT INTO registrant (name, phone, message, photo) VALUES ($1, $2, $3, $4) RETURNING id, name, phone, message, photo",
    )
    .bind(&data.name)
    .bind(&data.phone)
    .bind(&data.message)
    .bind(&imgur.data.link)
    .fetch_one(&stx.pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(row)))
}
