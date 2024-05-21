use reqwest::Error;
use scraper::{selectable::Selectable, Html, Selector};

#[derive(Default)]
pub struct Crawler {
    http: reqwest::blocking::Client,
    state: CrawlerState,
}

#[derive(Default)]
pub struct CrawlerState {
    url: String,
    content: Option<Html>,
    status_code: u16,
}

impl Crawler {
    pub fn goto(&mut self, url: &str) -> Result<(), Error> {
        let resp = self.http.get(url).send()?;
        self.state.status_code = resp.status().as_u16();
        self.state.url = resp.url().to_string();
        let content = resp.text()?;
        let doc = Html::parse_document(content.as_str());
        self.state.content = Some(doc);
        Ok(())
    }

    pub fn scrape(&self, selector: &str) -> String {
        let sel = Selector::parse(selector).unwrap();
        let elem = self.state.content.as_ref().unwrap().select(&sel).next();
        match elem {
            Some(e) => e.text().collect(),
            None => "".to_string(),
        }
    }

    pub fn status(&self) -> u16 {
        self.state.status_code
    }
}
