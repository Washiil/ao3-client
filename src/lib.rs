use scraper::{html, selectable::Selectable, Selector};
use reqwest;

pub mod work;

const BASE_URL: &str = "https://archiveofourown.org";

pub fn say_hello(name: &str) -> String {
    String::from(format!("Hello {}", name))
}

#[derive(Debug)]
enum ArchiveWarning {

}

struct Client {

}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn get_work() {
        let x = work::Work::new("58593067").await;
        
        match x {
            Ok(val) => todo!(),
            Err(e) => todo!()
        }
    }
}