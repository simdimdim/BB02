#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        crate::fetch("");
    }
}

pub fn fetch<'a>(_url: &'a str) {}
