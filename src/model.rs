use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Default)]
pub struct RegistrantDTO {
    pub name: String,
    pub phone: String,
    pub message: String,
    pub photo: Vec<u8>,
}

#[derive(Serialize, FromRow)]
pub struct Registrant {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub message: String,
    pub photo: String,
}

#[derive(Deserialize)]
pub struct ImgurResponse {
    pub status: u32,
    pub success: bool,
    pub data: ImgurData,
}

#[derive(Deserialize)]
pub struct ImgurData {
    pub title: String,
    pub description: String,
    pub link: String,
}

#[derive(Clone)]
pub struct Statex {
    pub pool: PgPool,
    pub token: String,
}
