//! The main backend server

mod db;
mod mqtt;

#[tokio::main]
async fn main() {
    let pool = db::init("flowers.db")
        .await
        .expect("Failed to initialize database");

    mqtt::init(pool).await;
}

