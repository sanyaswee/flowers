//! The main backend server

mod db;
mod mqtt;

#[tokio::main]
async fn main() {
    // Initialize DB connection
    let pool = db::init("flowers.db")
        .await
        .expect("Failed to initialize database");

    // Start MQTT listener
    mqtt::init(pool).await;
}

