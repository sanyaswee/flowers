//! The main backend server

mod db;
mod mqtt;
mod api;

#[tokio::main]
async fn main() {
    // Initialize DB connection
    let pool = db::init("flowers.db")
        .await
        .expect("Failed to initialize database");

    // Initialize MQTT
    let mqtt_client = mqtt::init(pool.clone()).await;

    // Start HTPP server
    let state = api::AppState {
        pool,
        mqtt_client
    };
    let app = api::app(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

