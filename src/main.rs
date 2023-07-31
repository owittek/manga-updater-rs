use std::error::Error;

use reqwest::Url;
use scraper::{Html, Selector};
use spinners::{Spinner, Spinners};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[allow(dead_code)]
#[derive(Debug)]
struct Manga {
    id: i16,
    title: String,
    urls: Vec<String>,
    chapter: i16,
    chapter_title: String,
}

trait MangaParser {
    fn parse(deserialized_html: &str, url: String) -> Result<Manga, Box<dyn Error>>;
}

struct ParseHelper {}

impl ParseHelper {
    fn get_first_number_from_string(string: &str) -> String {
        let mut number = String::new();
        for c in string.chars() {
            if c.is_ascii_digit() {
                number.push(c);
            } else if !number.is_empty() {
                break;
            }
        }
        number
    }

    fn get_string_post_separator(mut string: String, separator: char) -> String {
        string
            .split_off(string.find(separator).unwrap_or(string.len()))
            .trim()
            .to_string()
    }
}

struct AsuraScansParser {}

impl MangaParser for AsuraScansParser {
    fn parse(deserialized_html: &str, url: String) -> Result<Manga, Box<dyn Error>> {
        let html = Html::parse_document(deserialized_html);
        let raw_chapter_title = html
            .select(&Selector::parse("#chapterlist > ul")?)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join("");

        let title = html
            .select(&Selector::parse("h1")?)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join("");

        let chapter_number = ParseHelper::get_first_number_from_string(&raw_chapter_title);
        let chapter_title: String = ParseHelper::get_string_post_separator(raw_chapter_title, ':');
        Ok(Manga {
            id: -1,
            title: String::from(title.trim()),
            urls: vec![url],
            chapter: chapter_number.parse::<i16>().unwrap(),
            chapter_title,
        })
    }
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
#[tokio::main]
async fn main() {
    let db_url = get_db_url();
    let pool = get_db_client(db_url.as_str(), 5).await;

    let mangas = sqlx::query_as!(Manga, "SELECT * FROM manga")
        .fetch_all(&pool)
        .await
        .unwrap();
    println!("{:?}", mangas);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Error creating a request client");

    loop {
        for manga in &mangas {
            for url in &manga.urls {
                let res = client.get(url).send().await.expect("Error sending request");
                // let hmtl = &res.text().await.expect("Error reading response");
                println!("{:?}", &res);
            }
        }
    }
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
