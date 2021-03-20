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

    let mut table_vec: Vec<Vec<Vec<String>>> = vec![];

    for table in document.select(".table").unwrap() {
        for tbody in table.as_node().select("tbody").unwrap() {
            let mut tr_vec: Vec<Vec<String>> = vec![];

            for tr in tbody.as_node().select("tr").unwrap() {
                let mut td_vec: Vec<String> = vec![];

                for td in tr.as_node().select("td").unwrap() {
                    let td_child = td.as_node().first_child().unwrap();

                    if let Some(text) = td_child.as_text() {
                        td_vec.push(text.borrow().to_string());
                    }
                }

                tr_vec.push(td_vec);
            }

            table_vec.push(tr_vec);
        }
    }

    eprintln!("{:?}", table_vec);

    ApiStructure {
        functions: vec![],
        entities: vec![],
    }
}
