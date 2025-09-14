use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn new() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    let supabase_database_url = std::env::var("SUPABASE_DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&supabase_database_url)
        .await?;

    Ok(pool)
}
