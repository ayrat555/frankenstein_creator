use isahc::prelude::*;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;
use std::io;

pub struct Fetcher {
    url: String,
}

enum ParamType {
    Integer,
    String,
    Entity, // Array(ParamType)
}

pub struct Param {
    name: String,
    param_type: ParamType,
    description: String,
    required: bool,
}

pub struct Function {
    params: Vec<Param>,
    name: String,
}

pub struct Entity {
    fields: Vec<Param>,
    name: String,
}

pub struct ApiStructure {
    functions: Vec<Function>,
    entities: Vec<Entity>,
}

impl Fetcher {
    pub fn new(url: String) -> Self {
        Fetcher { url }
    }

    pub fn fetch(&self) -> Result<ApiStructure, String> {
        match isahc::get(&self.url) {
            Ok(mut response) => {
                if response.status() != 200 {
                    let msg = format!("Status code {}", response.status());

                    return Err(msg);
                }

                match response.text() {
                    Ok(text) => Ok(parse(text)),
                    Err(error) => fmt_error(error),
                }
            }
            Err(error) => fmt_error(error),
        }
    }
}

fn fmt_error<T: std::fmt::Debug>(error: T) -> Result<ApiStructure, String> {
    let msg = format!("{:?}", error);

    Err(msg)
}

fn parse(body: String) -> ApiStructure {
    let document = parse_html().one(body);
    eprintln!("{:?}", document);

    ApiStructure {
        functions: vec![],
        entities: vec![],
    }
}
