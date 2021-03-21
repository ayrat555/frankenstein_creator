use isahc::prelude::*;

pub struct Fetcher {
    url: String,
}

impl Fetcher {
    pub fn new(url: String) -> Self {
        Fetcher { url }
    }

    pub fn fetch(&self) -> Result<String, String> {
        match isahc::get(&self.url) {
            Ok(mut response) => {
                if response.status() != 200 {
                    let msg = format!("Status code {}", response.status());

                    return Err(msg);
                }

                match response.text() {
                    Ok(text) => Ok(text),
                    Err(error) => self.fmt_error(error),
                }
            }
            Err(error) => self.fmt_error(error),
        }
    }

    fn fmt_error<T: std::fmt::Debug>(&self, error: T) -> Result<String, String> {
        let msg = format!("{:?}", error);

        Err(msg)
    }
}
