use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod api;
mod detection;
mod domain;
mod state;

use api::{
    create_anomaly,
    create_event,
    get_anomalies,
    get_anomaly,
    get_event,
    get_events,
};
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Server starting up");

    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:4200"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .route("/events", post(create_event).get(get_events))
        .route("/events/{id}", get(get_event))
        .route("/anomalies", post(create_anomaly).get(get_anomalies))
        .route("/anomalies/{id}", get(get_anomaly))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = "0.0.0.0:3000"
        .parse()
        .expect("Invalid server address");

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind server address");

    println!("Listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}