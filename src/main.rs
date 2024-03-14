use axum::Router;
use model::Statex;
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;

mod controller;
mod model;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let token = secret_store.get("ACCESS_TOKEN").unwrap();
    let state = Statex { pool, token };

    let router = Router::new()
        .nest("/registrants", controller::registrant_router())
        .with_state(state);

    Ok(router.into())
}
