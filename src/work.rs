use crate::{ArchiveWarning, BASE_URL};
use reqwest::{self, Client};
use scraper::{selectable::Selectable, Html, Selector};
use thiserror::Error;

// Pre-parse selectors to avoid redundant parsing
lazy_static::lazy_static! {
    static ref TITLE_SELECTOR: Selector = Selector::parse("#workskin .preface.group h2.title.heading").unwrap();
    static ref AUTHOR_SELECTOR: Selector = Selector::parse("#workskin .preface.group h3.byline.heading").unwrap();
    static ref PUBLICATION_DATE_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.stats dd.published").unwrap();
    static ref UPDATED_DATE_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.stats dd.status").unwrap();
    static ref WORD_COUNT_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.stats dd.words").unwrap();
    static ref ARCHIVE_WARNINGS_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.warning.tags ul.commas li a.tag").unwrap();
    static ref TAGS_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.freeform.tags ul.commas li a.tag").unwrap();
    static ref CHARACTERS_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.character.tags ul.commas li a.tag").unwrap();
    static ref RELATIONSHIP_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.relationship.tags ul.commas li a.tag").unwrap();
    static ref CHAPTERS_SELECTOR: Selector = Selector::parse("#selected_id option").unwrap();
    static ref CONTENT_SELECTOR: Selector = Selector::parse("#workskin #chapters div.chapter div.userstuff.module p").unwrap();
    static ref HITS_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.stats dd.hits").unwrap();
    static ref LANGUAGE_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.language").unwrap();
    static ref RATING_SELECTOR: Selector = Selector::parse("div.wrapper dl.work.meta.group dd.rating.tags ul.commas li a.tag").unwrap();
    static ref SUMMARY_SELECTOR: Selector = Selector::parse("#workskin .preface.group div.summary.module blockquote.userstuff").unwrap();
    // Summary is broken up into multiple elements so we will have to figure out the best way to parse those
    // It appears things can use most markdown like <em> <br> <p> or <strong>
}

#[derive(Default, Debug)]
pub struct Chapter {
    name: String,
    id: String,
}

#[derive(Default, Debug)]
pub struct Tag {
    name: String,
    link: String,
}

#[derive(Debug)]
enum Content {
    Text(String),
    Bold(String),
    Italic(String),
}

#[derive(Default, Debug)]
struct Paragraph {
    content: Vec<Content>,
}

#[derive(Default, Debug)]
pub struct Work {
    title: String,
    date_published: String,
    date_updated: String,
    word_count: u32,
    author: String,
    archive_warnings: Vec<Tag>,
    tags: Vec<Tag>,
    characters: Vec<Tag>,
    relationships: Vec<Tag>,
    chapters: Vec<Chapter>,
    hits: u32,
    language: String,
    body: Vec<Paragraph>,
    ratings: Vec<Tag>,
}

#[derive(Error, Debug)]
pub enum WorkError {
    #[error("Element not found: {0}")]
    ElementNotFound(&'static str),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid Work ID or inaccessible work")]
    InvalidWorkId,

    #[error("Work {0} was not able to be parced")]
    InvalidFormat(&'static str),

    #[error("No items were found for the collection: {0}")]
    NoItemsFound(&'static str),
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

        output.title = Self::parse_inner_text_element(document, &TITLE_SELECTOR, "title")?;
        output.author = Self::parse_inner_text_element(document, &AUTHOR_SELECTOR, "author")?;
        output.date_published =
            Self::parse_inner_text_element(document, &PUBLICATION_DATE_SELECTOR, "published")?;
        output.date_updated =
            Self::parse_inner_text_element(document, &UPDATED_DATE_SELECTOR, "updated")?;
        output.language = Self::parse_inner_text_element(document, &LANGUAGE_SELECTOR, "language")?;
        output.word_count =
            Self::parse_inner_number_element(document, &WORD_COUNT_SELECTOR, "word_count")? as u32;
        output.hits = Self::parse_inner_number_element(document, &HITS_SELECTOR, "hits")? as u32;

        output.tags = Self::parse_tag_list(document, &TAGS_SELECTOR, "tags")?;
        output.characters = Self::parse_tag_list(document, &CHARACTERS_SELECTOR, "characters")?;
        output.archive_warnings =
            Self::parse_tag_list(document, &ARCHIVE_WARNINGS_SELECTOR, "warnings")?;
        output.relationships =
            Self::parse_tag_list(document, &RELATIONSHIP_SELECTOR, "relationships")?;
        output.ratings = Self::parse_tag_list(document, &RATING_SELECTOR, "ratings")?;
        output.chapters = Self::parse_chapters(document, &CHAPTERS_SELECTOR)?;
        output.body = Self::parse_paragraphs(document, &CONTENT_SELECTOR)?;

        // Add more parsing logic here as needed...

        Ok(output)
    }

    fn parse_paragraph(element: scraper::ElementRef) -> Result<Paragraph, WorkError> {
        // TODO: This has to work somewhat recursively to allow for nested types
        let mut content = Vec::new();
    
        for child in element.children() {
            if let Some(text) = child.value().as_text() {
                content.push(Content::Text(text.to_string().trim().to_string()));
            } else if let Some(element) = child.value().as_element() {
                let tag_name = element.name();
                match tag_name {
                    "strong" => {
                        println!("Strong: {:?}", element);
                    }
                    "em" => {
                        println!("Em: {:?}", element);
                    }
                    _ => {}
                }
            }
        }
    
        if content.is_empty() {
            return Err(WorkError::NoItemsFound("paragraph"));
        }
    
        Ok(Paragraph { content })
    }

    fn parse_paragraphs(document: &Html, selector: &Selector) -> Result<Vec<Paragraph>, WorkError> {
        let paragraphs = document
            .select(selector)
            .map(|p| Work::parse_paragraph(p))
            .collect::<Result<Vec<_>, _>>()?;
    
        if paragraphs.is_empty() {
            return Err(WorkError::NoItemsFound("paragraphs"));
        }
    
        Ok(paragraphs)
    }

    fn parse_chapters(document: &Html, selector: &Selector) -> Result<Vec<Chapter>, WorkError> {
        let chapters: Vec<Chapter> = document
            .select(selector)
            .filter_map(|element| {
                let name = element.text().collect::<String>().trim().to_string();
                let chapter_id = element.value().attr("value").map(|s| s.to_string());

                match chapter_id {
                    Some(id) => Some(Chapter { name, id }),
                    None => None,
                }
            })
            .collect::<Vec<Chapter>>();

        match chapters.is_empty() {
            true => Ok(vec![]),
            false => Ok(chapters),
        }
    }

    fn parse_tag_list(
        document: &Html,
        selector: &Selector,
        name: &'static str,
    ) -> Result<Vec<Tag>, WorkError> {
        let tags: Vec<Tag> = document
            .select(selector)
            .filter_map(|element| {
                let name = element.text().collect::<String>().trim().to_string();
                let link = element.value().attr("href").map(|s| s.to_string());

                match link {
                    Some(link) => Some(Tag { name, link }),
                    None => None,
                }
            })
            .collect();

        match tags.is_empty() {
            true => return Err(WorkError::NoItemsFound(name)),
            false => Ok(tags),
        }
    }

    fn parse_inner_number_element(
        document: &Html,
        selector: &Selector,
        name: &'static str,
    ) -> Result<i32, WorkError> {
        if let Some(val) = document.select(selector).next() {
            let str_num = val.text().collect::<String>().replace(",", "");
            return str_num
                .parse::<i32>()
                .map_err(|_| WorkError::InvalidFormat(name));
        }
        Err(WorkError::InvalidFormat(name))
    }

    fn parse_inner_text_element(
        document: &Html,
        selector: &Selector,
        name: &'static str,
    ) -> Result<String, WorkError> {
        document
            .select(selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .ok_or_else(|| WorkError::ElementNotFound(name))
    }

    fn parse_html_element() {
        // Must parse out inner html that contains
        // <em> <p> <strong> <br>
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn work_from_id() {
        let x = Work::new("55836682").await;

        assert!(x.is_ok());

        let x = x.unwrap();

        // println!("WORK:\n{:#?}", x);

        assert!(false)
    }
}
