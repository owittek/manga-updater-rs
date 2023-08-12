use reqwest::Url;
use scraper::{Html, Selector};
use thiserror::Error;

use crate::Manga;

use self::asura_scans::AsuraScansParser;

pub mod asura_scans;

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
    pub fn new(url: &str) -> Result<impl MangaParser, ParserError> {
        let url = Url::parse(url).expect("Error parsing URL");
        match url.host_str().unwrap() {
            "asura.gg" => Ok(AsuraScansParser),
            _ => Err(ParserError::HostNotFound),
        }
    }
}

pub(crate) trait MangaParser {
    fn parse(&self, deserialized_html: &str, url: &str) -> Result<Manga, ParserError>;
}

pub struct ParseHelper;

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
