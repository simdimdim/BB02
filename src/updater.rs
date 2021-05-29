use crate::{downloader::Downloader, library::Library};
use reqwest::Url;
use select::document::Document;

pub struct Updater {
    _dl: Downloader,
    lib: Library,
}
impl Updater {
    pub async fn refresh(&mut self) {
        self.lib
            .books
            .iter()
            .map(|(_, (b, s))| (b, s))
            .for_each(|(b, _s)| {
                // let site = self.lib.get_site(&s[0]);
                // self.dl.download(site);
                #[allow(path_statements)]
                {
                    b;
                }
            });
    }

    #[allow(dead_code)]
    fn get_images(_html: Document) {}

    #[allow(dead_code)]
    fn get_next(
        _html: Document,
        _url: Url,
    ) {
    }

    #[allow(dead_code)]
    fn get_index(
        _html: Document,
        _url: Url,
    ) {
    }
}
