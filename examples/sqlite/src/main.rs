use anyhow::Result;
use apalis::{layers::TraceLayer, prelude::*, sqlite::SqliteStorage};
use chrono::Utc;

use email_service::{send_email, Email};

async fn produce_jobs(storage: &SqliteStorage<Email>) -> Result<()> {
    let mut storage = storage.clone();
    for i in 0..2 {
        storage
            .schedule(
                Email {
                    to: format!("test{i}@example.com"),
                    text: "Test background job from apalis".to_string(),
                    subject: "Background email job".to_string(),
                },
                Utc::now() + chrono::Duration::seconds(i),
            )
            .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug,sqlx::query=error");
    tracing_subscriber::fmt::init();

    let sqlite: SqliteStorage<Email> = SqliteStorage::connect("sqlite::memory:").await?;
    // Do migrations: Mainly for "sqlite::memory:"
    sqlite
        .setup()
        .await
        .expect("unable to run migrations for sqlite");

    // This can be in another part of the program
    produce_jobs(&sqlite).await?;

    Monitor::new()
        .register_with_count(2, move |c| {
            WorkerBuilder::new(format!("tasty-banana-{c}"))
                .layer(TraceLayer::new())
                .with_storage(sqlite.clone())
                .build_fn(send_email)
        })
        .run()
        .await?;
    Ok(())
}