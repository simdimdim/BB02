#![feature(with_options)]
#![feature(bool_to_option)]
#![feature(destructuring_assignment)]
#![feature(slice_pattern)]

pub mod library;
pub mod retriever;
pub mod source;
pub mod update;

pub const CACHE: &str = "./.cache";
pub const TEST: &str = "https://readmanganato.com/manga-la988983";

#[tokio::test]
async fn manager_refresh() {
    use crate::update::Manager;

    let mut manager = Manager::default();
    manager.add_book(None, TEST.to_string().into()).await;
    println!("{}", manager.refresh().await);
}
