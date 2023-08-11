use reqwest::Url;
use scraper::{Html, Selector};
use spinners::{Spinner, Spinners};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use thiserror::Error;

#[derive(Debug)]
struct Manga {
    id: Option<i16>,
    title: String,
    image_url: Option<String>,
    urls: Vec<String>,
    chapter: i16,
    chapter_title: Option<String>,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("DOM element not found")]
    ElementNotFound,
    #[error("attribute of element not found")]
    AttributeNotFound,
    #[error("host is either not supported or not found")]
    HostNotFound,
}

impl dyn MangaParser {
    fn new(url: &str) -> Result<impl MangaParser, ParserError> {
        let url = Url::parse(url).expect("Error parsing URL");
        match url.host_str().unwrap() {
            "asura.gg" => Ok(AsuraScansParser),
            _ => Err(ParserError::HostNotFound),
        }
    }
}

trait MangaParser {
    fn parse(&self, deserialized_html: &str, url: &str) -> Result<Manga, ParserError>;
}

struct ParseHelper;

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

    fn get_string_post_separator(mut string: String, separator: char) -> Option<String> {
        let index = string.find(separator)?;
        match string.split_off(index).trim().to_string() {
            s if s.is_empty() => None,
            s => Some(s),
        }
    }

    fn get_text_from_first_result(
        deserialized_html: &str,
        selector: &Selector,
    ) -> Result<String, ParserError> {
        let html = Html::parse_document(deserialized_html);
        let first_el = match html.select(selector).next() {
            Some(e) => e,
            None => return Err(ParserError::ElementNotFound),
        };

        let text = first_el
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        Ok(text)
    }

    fn get_src_from_first_result(
        deserialized_html: &str,
        selector: &Selector,
    ) -> Result<String, ParserError> {
        let html = Html::parse_document(deserialized_html);
        let first_el = match html.select(selector).next() {
            Some(e) => e,
            None => return Err(ParserError::ElementNotFound),
        };

        match first_el.value().attr("src") {
            Some(image_url) => Ok(image_url.to_string()),
            None => Err(ParserError::AttributeNotFound),
        }
    }
}

struct AsuraScansParser;

impl MangaParser for AsuraScansParser {
    fn parse(&self, deserialized_html: &str, url: &str) -> Result<Manga, ParserError> {
        let raw_chapter_title = ParseHelper::get_text_from_first_result(
            deserialized_html,
            &Selector::parse("#chapterlist > ul").unwrap(),
        )?;

        let title = ParseHelper::get_text_from_first_result(
            deserialized_html,
            &Selector::parse("h1").unwrap(),
        )?;

        let chapter_number = ParseHelper::get_first_number_from_string(&raw_chapter_title);
        let chapter_title = ParseHelper::get_string_post_separator(raw_chapter_title, ':');
        let image_url = match ParseHelper::get_src_from_first_result(
            deserialized_html,
            &Selector::parse("img.attachment-.size-.wp-post-image").unwrap(),
        ) {
            Ok(image_url) => Some(image_url),
            Err(e) => {
                println!("error getting the image for {}: {}", title, e);
                None
            }
        };

        Ok(Manga {
            id: None,
            title,
            image_url,
            urls: vec![url.to_string()],
            chapter: chapter_number
                .parse::<i16>()
                .expect("error parsing chapter number"),
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

    // let mangas = sqlx::query_as!(Manga, "SELECT * FROM manga")
    //     .fetch_all(&pool)
    //     .await
    //     .unwrap();

    let client = reqwest::Client::builder()
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
