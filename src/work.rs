use crate::{ArchiveWarning, BASE_URL};
use reqwest::{self, Client};
use scraper::{Html, Selector};
use thiserror::Error;

// Pre-parse selectors to avoid redundant parsing
lazy_static::lazy_static! {
    static ref TITLE_SELECTOR: Selector = Selector::parse("#workskin .preface.group h2.title.heading").unwrap();
    static ref AUTHOR_SELECTOR: Selector = Selector::parse("#workskin .preface.group h3.byline.heading").unwrap();
}

#[derive(Default)]
pub struct Work {
    title: String,
    date: String,
    words: u32,
    author: String,
    archive_warnings: Vec<ArchiveWarning>,
    tags: Vec<String>,
    characters: Vec<String>,
    relationships: Vec<String>,
    current_chapter: u32,
    total_chapters: u32,
    hits: u32,
    language: String,
    rating: String,
}

#[derive(Error, Debug)]
pub enum WorkError {
    #[error("Element not found: {0}")]
    ElementNotFound(&'static str),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid Work ID or inaccessible work")]
    InvalidWorkId,
}

impl Work {
    pub async fn new(work_id: &str) -> Result<Self, WorkError> {
        let client = Client::new();
        let url = format!("{}/works/{}?view_adult=true", BASE_URL, work_id);
        let content_body = Self::fetch_content(&client, &url).await?;

        let document = Html::parse_document(&content_body);
        Self::parse_document(&document)
    }

    async fn fetch_content(client: &Client, url: &str) -> Result<String, WorkError> {
        let response = client.get(url).send().await?;
        let content_body = response.text().await?;

        if content_body.contains("system errors error-404 region") {
            return Err(WorkError::InvalidWorkId);
        }

        Ok(content_body)
    }

    fn parse_document(document: &Html) -> Result<Work, WorkError> {
        let mut output = Work::default();

        output.title = Self::parse_element(document, &TITLE_SELECTOR, "title")?;
        output.author = Self::parse_element(document, &AUTHOR_SELECTOR, "author")?;
        
        // Add more parsing logic here as needed...

        Ok(output)
    }

    fn parse_element(document: &Html, selector: &Selector, name: &'static str) -> Result<String, WorkError> {
        document
            .select(selector)
            .next()
            .map(|element| element.text().collect::<String>())
            .ok_or_else(|| WorkError::ElementNotFound(name))
    }
}