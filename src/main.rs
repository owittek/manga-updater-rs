pub mod parser;

use crate::parser::MangaParser;
use parser::ParserError;
use reqwest::{Client, Url};
use spinners::{Spinner, Spinners};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Debug)]
struct Manga {
    id: Option<i16>,
    title: String,
    image_url: Option<String>,
    urls: Vec<String>,
    chapter: i16,
    chapter_title: Option<String>,
}

/*
* Add new manga: new URL
* Add URL to existing Manga: ID, URL
* Get table of all manga: ID, Title, URL, Chapter
* Delete Manga by ID
* Remove URL from Manga: ID, URL or host
*/

/*
* Dead links: If not 200, remove URL from Manga & notify user, add to dead links list
*/

// TODO: URL & timeout should be configurable via CLI
#[allow(unused_variables)]
#[tokio::main]
async fn main() {
    let db_url = get_db_url();
    let pool = get_db_client(db_url.as_str(), 5).await;

    /*
    let mangas = sqlx::query_as!(Manga, "SELECT * FROM manga")
        .fetch_all(&pool)
        .await
        .unwrap();
    */

    let client: Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Error creating a request client");

    let test_url = "https://asura.gg/manga/0223090894-return-of-the-mount-hua-sect/";
    let res = client
        .get(test_url)
        .send()
        .await
        .expect("Error sending request");
    let html = &res.text().await.expect("Error reading response");
    let parsed_manga = <dyn MangaParser>::new(test_url)
        .unwrap()
        .parse(html, test_url)
        .unwrap();
    let mut message = format!("{} - Chapter {}", parsed_manga.title, parsed_manga.chapter);
    if let Some(chapter_title) = parsed_manga.chapter_title {
        message.push_str(&format!(": {}", chapter_title));
    }
    println!("{}", message);

    /*
    loop {
        for manga in &mangas {
            for url in &manga.urls {
                let res = client.get(url).send().await.expect("Error sending request");
                let hmtl = &res.text().await.expect("Error reading response");
                let parsed_manga = AsuraScansParser::parse(hmtl, url.to_string()).unwrap();
                println!("{:?}", parsed_manga);
            }
        }
    }
    */
}

fn get_db_url() -> String {
    dotenvy::dotenv().ok();
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let parsed_url = Url::parse(db_url.as_str()).expect("DATABASE_URL must be a valid URL");
    let mut is_invalid_url = false;

    if parsed_url.scheme() != "postgres" {
        eprintln!("DATABASE_URL must be a postgres URL");
        is_invalid_url = true;
    }
    if parsed_url.username().is_empty() {
        eprintln!("DATABASE_URL must contain a username");
        is_invalid_url = true;
    }
    if parsed_url.password().is_none() {
        eprintln!("DATABASE_URL must contain a password");
        is_invalid_url = true
    }
    if parsed_url.port().is_none() {
        eprintln!("DATABASE_URL must contain a port");
        is_invalid_url = true
    }
    if is_invalid_url {
        std::process::exit(1);
    }
    db_url
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
        Err(e) => {
            sp.stop_with_message("✖ Database connection failed!".into());
            panic!("{}", e)
        }
    }
}

#[allow(dead_code)]
async fn add_manga(
    client: &reqwest::Client,
    pool: &Pool<Postgres>,
    url: &str,
) -> Result<Manga, ParserError> {
    let res = client.get(url).send().await.expect("Error sending request");
    let html = res.text().await.expect("Error reading response");
    let mut manga = <dyn MangaParser>::new(url)?.parse(&html, url)?;
    let res = sqlx::query!(
        r#"
    INSERT INTO manga (title, image_url, urls, chapter, chapter_title)
    VALUES ($1, $2, $3, $4, $5)
    RETURNING id;"#,
        manga.title,
        manga.image_url,
        &manga.urls,
        manga.chapter,
        manga.chapter_title
    )
    .fetch_one(pool)
    .await;

    manga.id = Some(res.unwrap().id);
    Ok(manga)
}
