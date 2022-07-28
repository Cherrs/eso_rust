#![feature(string_remove_matches)]
use std::{collections::HashMap, result};

use category::Category;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use crate::addon::{Addon, Page};
mod addon;
mod category;

pub struct Esoui {
    client: Client,
    url: String,
}

impl Esoui {
    pub fn new(url: String) -> Self {
        Esoui {
            client: reqwest::Client::new(),
            url,
        }
    }

    pub async fn cat_get(&self) -> Result<Vec<Category>, EsouiError> {
        let url = format!("{}{}", self.url, "/addons.php");
        let html = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&html);
        let cat_selector = Selector::parse("#col1 > div > div > div:nth-child(2) > div").unwrap();
        let cat_raw = document.select(&cat_selector);
        let mut result = Vec::new();
        for i in cat_raw {
            let files_count = text_get(&i, "div.subtitle > span.filecount");
            if files_count.is_empty() {
                continue;
            }
            let mut files_count = text_get(&i, "div.subtitle > span.filecount")[0].to_string();
            files_count.remove_matches(" files");
            let files_count: i32 = files_count.parse().unwrap();
            result.push(Category {
                name: text_get(&i, "div.subtitle > a")[0].to_string(),
                files_count,
                url: attrs_get(&i, "div.subtitle > a")
                    .get("href")
                    .unwrap()
                    .to_string(),
                icon: String::from("123"),
            })
        }
        Ok(result)
    }

    pub async fn latest_200_orderbydl_get(&self, page: usize) -> Result<Vec<Addon>, EsouiError> {
        if !(1..8).contains(&page) {
            return Err(EsouiError::InvalidParameter(String::from("页数必须在1-8")));
        }
        let mut url = self.url.clone();
        url.push_str("/downloads/latest.php?sb=dlcount&so=desc&sh=full&pt=f&page=");
        url.push_str(&page.to_string());
        let html = self.client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&html);
        let table_selector = Selector::parse("#innerpage > table.tborder > tbody > tr").unwrap();
        let table = document.select(&table_selector);
        let mut result = Vec::new();
        for i in table {
            let _name = text_get(&i, "td:nth-child(2)");
            let name = _name[0];
            let url = attrs_get(&i, "td:nth-child(2) > a");
            let _last_update = text_get(&i, "td:nth-child(6)");
            result.push(Addon {
                name: name.to_string(),
                author: inner_html_get(i, "td:nth-child(3)"),
                size: inner_html_get(i, "td:nth-child(4)"),
                downloads: inner_html_get(i, "td:nth-child(5)"),
                last_update: format!("{} {}", _last_update[5], _last_update[6]),
                url: url.get("href").unwrap().to_string(),
                page: Page {
                    count: 200,
                    page: 1,
                    page_count: 25,
                },
            });
        }
        Ok(result)
    }
    async fn selector_get(
        &self,
        document: &Html,
        selector_str: &str,
    ) -> Result<Vec<String>, EsouiError> {
        let selector = Selector::parse(selector_str).unwrap();
        let s = document.select(&selector);
        Ok(document.select(&selector).map(|f| f.inner_html()).collect())
    }
}

fn inner_html_get(elem: ElementRef, selectors: &str) -> String {
    let inner_selector = Selector::parse(selectors).unwrap();
    let item = elem.select(&inner_selector).next().unwrap();
    item.text().next().unwrap().to_string()
}
fn text_get<'a>(elem: &'a ElementRef, selectors: &str) -> Vec<&'a str> {
    let inner_selector = Selector::parse(selectors).unwrap();
    let item = elem.select(&inner_selector).next().unwrap();
    item.text().collect()
}
fn attrs_get<'a>(elem: &'a ElementRef, selectors: &str) -> HashMap<&'a str, &'a str> {
    let inner_selector = Selector::parse(selectors).unwrap();
    let item = elem.select(&inner_selector).next().unwrap();
    item.value().attrs().collect()
}

impl Default for Esoui {
    fn default() -> Self {
        Self::new(String::from("https://www.esoui.com"))
    }
}

#[derive(Error, Debug)]
pub enum EsouiError {
    #[error("无效参数,{0}")]
    InvalidParameter(String),
    #[error("data store disconnected")]
    RequestError(#[from] reqwest::Error),
    #[error("unknown data store error")]
    Unknown,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn latest_200_orderbydl_get() -> Result<(), EsouiError> {
        let result = Esoui::default();
        result.latest_200_orderbydl_get(8).await.unwrap();
        Ok(())
    }
    #[tokio::test]
    async fn cat_get() -> Result<(), EsouiError> {
        let eso = Esoui::default();
        let r = eso.cat_get().await?;
        println!("{:?}", r);
        Ok(())
    }
}
