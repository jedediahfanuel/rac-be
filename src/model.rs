use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Default, Deserialize)]
pub struct RegistrantFormData {
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
    pub photo: Vec<u8>
}

#[derive(Serialize, FromRow)]
pub struct RegistrantString {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub message: String,
    pub photo: String
}

#[derive(Serialize, FromRow)]
pub struct Response {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub message: String
}
