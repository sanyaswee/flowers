//! The main backend server

mod api;
mod auto_watering;
mod db;
mod mqtt;

#[tokio::main]
async fn main() {
    // Initialize DB connection
    let pool = db::init("flowers.db")
        .await
        .expect("Failed to initialize database");

    // Initialize MQTT
    let mqtt_client = mqtt::init(pool.clone()).await;

    // Start the auto watering task
    auto_watering::spawn(pool.clone(), mqtt_client.clone());

    // Start HTPP server
    let state = api::AppState { pool, mqtt_client };
    let app = api::app(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
