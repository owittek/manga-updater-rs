use scraper::Selector;

use crate::Manga;

use super::{MangaParser, ParseHelper, ParserError};

pub struct AsuraScansParser;

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
