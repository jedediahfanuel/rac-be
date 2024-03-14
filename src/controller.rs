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
        Ok(registrants) => {
            let row = registrants
                .into_iter()
                .map(|reg| RegistrantResponse {
                    id: reg.id,
                    name: reg.name,
                    phone: reg.phone,
                    message: reg.message,
                    photo: reg.photo,
                })
                .collect::<Vec<RegistrantResponse>>();
            Ok((StatusCode::OK, Json(row)))
        }
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
            "photo" => data.photo = field.bytes().await.unwrap(),
            _ => {}
        }
    }

    let client = reqwest::Client::new();
    let mut form_data = Form::new();

    form_data = form_data.text("title", format!("{}", data.name));
    form_data = form_data.text(
        "description",
        format!("{} --- {}", data.phone, data.message),
    );
    form_data = form_data.part(
        "image",
        reqwest::multipart::Part::bytes(data.photo.to_vec()),
    );

    let imgur_response = client
        .post("https://api.imgur.com/3/image")
        .header("Authorization", format!("Bearer {}", stx.token.clone()))
        .multipart(form_data)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let imgur: ImgurResponse = serde_json::from_str(&imgur_response).unwrap();

    if !imgur.success {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Upload photo failed".to_string(),
        ));
    }

    match sqlx::query_as::<_, Response>(
        "INSERT INTO registrant (name, phone, message, photo) VALUES ($1, $2, $3, $4) RETURNING id, name, phone, message",
    )
    .bind(&data.name)
    .bind(&data.phone)
    .bind(&data.message)
    .bind(&imgur.data.link)
    .fetch_one(&stx.pool)
    .await
    {
        Ok(row) => Ok((StatusCode::CREATED, Json(row))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}
