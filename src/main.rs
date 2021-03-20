use frankenstein_creator::fetcher::Fetcher;

fn main() {
    Fetcher::new("https://core.telegram.org/bots/api".to_string()).fetch();
}
