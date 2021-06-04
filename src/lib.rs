#![feature(with_options)]
#![feature(bool_to_option)]
#![feature(destructuring_assignment)]
#![feature(slice_pattern)]

pub mod library;
pub mod retriever;
pub mod source;
pub mod update;

pub static CACHE: &str = "./.cache";
pub static TEST: &str = "https://readmanganato.com/manga-jd986360/chapter-30";
