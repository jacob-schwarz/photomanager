mod model;
mod index;
mod cli; 

use simple_logger::SimpleLogger;
use sqlx::sqlite::SqlitePool;
use std::env;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    SimpleLogger::new().init().unwrap();

    let pool = SqlitePool::connect(env!("DATABASE_URL")).await?;

    cli::init(&pool).await?;

    Ok(())
}
