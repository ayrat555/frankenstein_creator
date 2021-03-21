use frankenstein_creator::fetcher::Fetcher;
use frankenstein_creator::parser::Parser;

fn main() {
    let html = Fetcher::new("https://core.telegram.org/bots/api".to_string())
        .fetch()
        .unwrap();

    Parser::new(html).parse();
}
