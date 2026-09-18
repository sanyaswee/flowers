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

    // Start MQTT listener
    let mqtt_pool = pool.clone();
    tokio::spawn(async move {
        mqtt::init(mqtt_pool).await;
    });

    // Start HTPP server
    let app = api::app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

