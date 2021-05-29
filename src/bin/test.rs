#![allow(unused_imports)]

use reqwest::Client;
use select::{
    document::Document,
    predicate::{Child, Name, Text},
};

static TEST: &str = "https://readmanganato.com/manga-lw988479/chapter-9";

async fn _download() {
    let html = Client::new()
        .get(TEST.to_string())
        .send()
        .await
        .ok()
        .unwrap()
        .text()
        .await
        .ok()
        .unwrap();
    std::fs::write("example.html", html).expect("To succeed");
}
async fn test() {
    let html = std::fs::read_to_string("example.html").unwrap();
    let doc = Document::from(html.as_str());
    dbg!(
        // "{}",
        doc.select(Child(Name("a"), Text))
            .filter(|a| a.text().contains("NEXT"))
            .map(|a| a.parent().unwrap().attr("href").unwrap().to_string())
            .next()
    );
}
#[tokio::main]
async fn main() {
    // _download().await;
    test().await;
}
