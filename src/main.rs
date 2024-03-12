use axum::Router;
use sqlx::PgPool;
use shuttle_runtime::CustomError;
use shuttle_axum::ShuttleAxum;

mod controller;
mod model;

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let router = Router::new()
        .nest("/registrants", controller::registrant_router())
        .with_state(pool);

    Ok(router.into())
}
