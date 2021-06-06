#![allow(unused_imports)]
use ehound::{update::Manager, TEST};
use reqwest::Client;
use select::{document::Document, predicate::*};

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

#[tokio::main]
async fn main() {
    // _download().await;

    let html = std::fs::read_to_string("example.html").unwrap();
    let doc = Document::from(html.as_str());
    dbg!(
        // "{}",
        doc.select(Descendant(
            Name("div"),
            Or(Name("p"), Or(Name("table"), Name("ul")))
        ))
        .map(|a| a.select(Name("a")).into_selection())
        .max_by(|a, b| a.len().cmp(&b.len()))
        .unwrap()
        .iter()
        .filter_map(|a| a.attr("href"))
        .map(|a| a.to_string())
        .collect::<Vec<String>>()
    );
}
