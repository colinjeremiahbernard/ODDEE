mod api;
mod domain;
mod state;

use axum::{Router, routing::post};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;

use crate::{api::events::create_event, state::AppState};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/events", post(create_event))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState { pool };
    let app = app(state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind address");

    println!("Listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("Server failed");
}
