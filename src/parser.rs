use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;
use kuchiki::{ElementData, NodeDataRef, NodeRef};

pub struct Parser {
    html: String,
}

impl Parser {
    pub fn new(html: String) -> Self {
        Parser { html }
    }

    pub fn parse(&self) -> Vec<Vec<Vec<String>>> {
        let document = parse_html().one(self.html.clone());

        let mut table_vec: Vec<Vec<Vec<String>>> = vec![];

        for table in document.select(".table").unwrap() {
            let (description, name_node): (String, NodeRef) = self.parse_description(&table);
            let name: String = self.parse_name(name_node);

            self.parse_table(table, &mut table_vec);
        }

        eprintln!("{:?}", table_vec[0]);

        // ApiStructure {
        //     functions: vec![],
        //     entities: vec![],
        // }

        table_vec
    }

    fn parse_description(&self, table_node: &NodeDataRef<ElementData>) -> (String, NodeRef) {
        let mut current_node = table_node.as_node().previous_sibling().unwrap();

        let mut name_node: Option<NodeRef> = None;
        let mut description = "".to_string();

        while name_node.is_none() {
            self.get_visible_text(&current_node, &mut description);

            current_node = current_node.previous_sibling().unwrap();

            if let Some(element) = current_node.as_element() {
                if &element.name.local == "h4" {
                    name_node = Some(current_node.clone());
                }
            }
        }

        (description, name_node.unwrap())
    }

    fn parse_name(&self, name_node: NodeRef) -> String {
        let mut name = "".to_string();

        self.get_visible_text(&name_node, &mut name);

        name
    }

    fn parse_table(&self, table: NodeDataRef<ElementData>, table_vec: &mut Vec<Vec<Vec<String>>>) {
        for tbody in table.as_node().select("tbody").unwrap() {
            let mut tr_vec: Vec<Vec<String>> = vec![];

            for tr in tbody.as_node().select("tr").unwrap() {
                let mut td_vec: Vec<String> = vec![];

                for td in tr.as_node().select("td").unwrap() {
                    let mut text: String = "".to_string();

                    self.get_visible_text(td.as_node(), &mut text);

                    td_vec.push(text);
                }

                tr_vec.push(td_vec);
            }

            table_vec.push(tr_vec);
        }
    }

    fn get_visible_text(&self, root: &NodeRef, processed_text: &mut String) {
        for child in root.children() {
            if let Some(el) = child.as_element() {
                let tag_name = &el.name.local;
                if tag_name == "script" || tag_name == "style" || tag_name == "noscript" {
                    return;
                }
                self.get_visible_text(&child, processed_text);
            } else if let Some(text_node) = child.as_text() {
                let text = text_node.borrow();
                processed_text.push_str(&text);
            }
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

        let result = Parser::new(html_table).parse();

        assert_eq!(1, result.len());
        assert_eq!(14, result[0].len());
        assert_eq!(vec!["update_id".to_string(), "Integer".to_string(), "The update\'s unique identifier. Update identifiers start from a certain positive number and increase sequentially. This ID becomes especially handy if you\'re using Webhooks, since it allows you to ignore repeated updates or to restore the correct update sequence, should they get out of order. If there are no new updates for at least a week, then identifier of the next update will be chosen randomly instead of sequentially.".to_string()], result[0][0]);
    }
}
