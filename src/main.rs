use reqwest::Url;
use spinners::{Spinner, Spinners};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

struct Manga {
    id: i32,
    title: String,
    urls: Vec<Url>,
    chapter: u16,
}

/*
* Add new manga: new URL
* Add URL to existing Manga: ID, URL
* Get table of all manga: ID, Title, URL, Chapter
* Delete Manga by ID
* Remove URL from Manga: ID, URL or host
*/

/*
* DB: ID, title, urls, chapter
*/

// TODO: URL & timeout should be configurable via CLI
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = get_db_client(db_url.as_str(), 5).await;

    let mangas = sqlx::query!("SELECT * FROM manga")
        .fetch_all(&pool)
        .await
        .unwrap();
    println!("{:?}", mangas);
}

async fn get_db_client(db_url: &str, pool_size: u32) -> Pool<Postgres> {
    let mut sp = Spinner::new(
        Spinners::Dots,
        "Waiting for the database to be reachable...".into(),
    );
    match PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(db_url)
        .await
    {
        Ok(pool) => {
            sp.stop_with_message("✔ Database connection established!".into());
            pool
        }
        Err(e) => panic!("Error connecting to database: {}", e),
    }
}
