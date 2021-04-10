use frankenstein_creator::fetcher::Fetcher;
use frankenstein_creator::generator::Generator;
use frankenstein_creator::parser::Parser;

fn main() {
    let html = Fetcher::new("https://core.telegram.org/bots/api".to_string())
        .fetch()
        .unwrap();

    let api_structure = Parser::new(html).parse();

    let mut generator = Generator::new(api_structure);

    generator.generate_entity_data();

    println!("{}", generator.to_string());
}
