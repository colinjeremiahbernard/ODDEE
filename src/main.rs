use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod api;
mod detection;
mod domain;
mod state;

use api::{create_anomaly, create_event, get_anomalies, get_anomaly, get_event, get_events};
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState { pool };

    let app = Router::new()
        .route("/events", post(create_event).get(get_events))
        .route("/events/{id}", get(get_event))
        .route("/anomalies", post(create_anomaly).get(get_anomalies))
        .route("/anomalies/{id}", get(get_anomaly))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
