use isahc::prelude::*;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
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

    pub fn fetch(&self) -> Result<Vec<Vec<Vec<String>>>, String> {
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

fn fmt_error<T: std::fmt::Debug>(error: T) -> Result<Vec<Vec<Vec<String>>>, String> {
    let msg = format!("{:?}", error);

    Err(msg)
}

fn parse(body: String) -> Vec<Vec<Vec<String>>> {
    let document = parse_html().one(body);

    let mut table_vec: Vec<Vec<Vec<String>>> = vec![];

    for table in document.select(".table").unwrap() {
        for tbody in table.as_node().select("tbody").unwrap() {
            let mut tr_vec: Vec<Vec<String>> = vec![];

            for tr in tbody.as_node().select("tr").unwrap() {
                let mut td_vec: Vec<String> = vec![];

                for td in tr.as_node().select("td").unwrap() {
                    let mut text: String = "".to_string();

                    get_visible_text(td.as_node(), &mut text);

                    td_vec.push(text);
                }

                tr_vec.push(td_vec);
            }

            table_vec.push(tr_vec);
        }
    }

    eprintln!("{:?}", table_vec[0]);

    // ApiStructure {
    //     functions: vec![],
    //     entities: vec![],
    // }

    table_vec
}

fn get_visible_text(root: &NodeRef, processed_text: &mut String) {
    for child in root.children() {
        if let Some(el) = child.as_element() {
            let tag_name = &el.name.local;
            if tag_name == "script" || tag_name == "style" || tag_name == "noscript" {
                return;
            }
            get_visible_text(&child, processed_text);
        } else if let Some(text_node) = child.as_text() {
            let text = text_node.borrow();
            processed_text.push_str(&text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn it_parses_table() {
        let html_table = fs::read_to_string("./test/support/table_example.html").unwrap();

        let result = parse(html_table);

        assert_eq!(1, result.len());
        assert_eq!(14, result[0].len());
        assert_eq!(vec!["update_id".to_string(), "Integer".to_string(), "The update\'s unique identifier. Update identifiers start from a certain positive number and increase sequentially. This ID becomes especially handy if you\'re using Webhooks, since it allows you to ignore repeated updates or to restore the correct update sequence, should they get out of order. If there are no new updates for at least a week, then identifier of the next update will be chosen randomly instead of sequentially.".to_string()], result[0][0]);
    }
}
