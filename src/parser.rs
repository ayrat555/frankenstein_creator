use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;
use kuchiki::{ElementData, NodeDataRef, NodeRef};

#[derive(Debug, PartialEq)]
pub struct Param {
    name: String,
    param_type: String,
    description: String,
    required: bool,
}

#[derive(Debug)]
pub struct Function {
    params: Vec<Param>,
    description: String,
    name: String,
}

#[derive(Debug)]
pub struct Entity {
    fields: Vec<Param>,
    description: String,
    name: String,
}

#[derive(Debug)]
pub struct ApiStructure {
    functions: Vec<Function>,
    entities: Vec<Entity>,
}

pub struct Parser {
    html: String,
}

impl Parser {
    pub fn new(html: String) -> Self {
        Parser { html }
    }

    pub fn parse(&self) -> ApiStructure {
        let parsed_html = self.parse_html();

        self.create_api_structure(parsed_html)
    }

    fn parse_html(&self) -> Vec<(String, String, Vec<Vec<String>>)> {
        let mut table_vec: Vec<(String, String, Vec<Vec<String>>)> = vec![];
        let document = parse_html().one(self.html.clone());

        for table in document.select(".table").unwrap() {
            let (description, name_node): (String, NodeRef) = self.parse_description(&table);
            let name: String = self.parse_name(name_node);

            let parsed_table = self.parse_table(table);

            table_vec.push((name, description, parsed_table));
        }

        table_vec
    }

    fn create_api_structure(
        &self,
        parsed_html_tables: Vec<(String, String, Vec<Vec<String>>)>,
    ) -> ApiStructure {
        let mut functions: Vec<Function> = vec![];
        let mut entities: Vec<Entity> = vec![];

        for (name, description, table) in parsed_html_tables {
            if table[0].len() == 3 {
                let entity = self.create_entity(table, name, description);

                entities.push(entity);
            } else {
                let function = self.create_function(table, name, description);

                functions.push(function);
            }
        }

        ApiStructure {
            functions: functions,
            entities: entities,
        }
    }

    fn create_entity(&self, table: Vec<Vec<String>>, name: String, description: String) -> Entity {
        let fields = table
            .into_iter()
            .map(|row| Param {
                name: row[0].clone(),
                param_type: row[1].clone(),
                description: row[2].clone(),
                required: !row[2].starts_with("Optional"),
            })
            .collect::<Vec<Param>>();

        Entity {
            name,
            description,
            fields,
        }
    }

    fn create_function(
        &self,
        table: Vec<Vec<String>>,
        name: String,
        description: String,
    ) -> Function {
        let params = table
            .into_iter()
            .map(|row| Param {
                name: row[0].clone(),
                param_type: row[1].clone(),
                description: row[3].clone(),
                required: row[2] == "Yes".to_string(),
            })
            .collect::<Vec<Param>>();

        Function {
            name,
            description,
            params,
        }
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

    fn parse_table(&self, table: NodeDataRef<ElementData>) -> Vec<Vec<String>> {
        let mut tr_vec: Vec<Vec<String>> = vec![];

        for tbody in table.as_node().select("tbody").unwrap() {
            for tr in tbody.as_node().select("tr").unwrap() {
                let mut td_vec: Vec<String> = vec![];

                for td in tr.as_node().select("td").unwrap() {
                    let mut text: String = "".to_string();

                    self.get_visible_text(td.as_node(), &mut text);

                    td_vec.push(text);
                }

                tr_vec.push(td_vec);
            }
        }

        tr_vec
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
    fn it_parses_entity_table() {
        let html_table = fs::read_to_string("./test/support/entity_table_example.html").unwrap();

        let result = Parser::new(html_table).parse();

        assert_eq!(0, result.functions.len());
        assert_eq!(1, result.entities.len());

        let entity = &result.entities[0];

        assert_eq!("Update".to_string(), entity.name);
        assert_eq!("This object represents an incoming update.At most one of the optional parameters can be present in any given update.".to_string(), entity.description);

        let expected_params = vec![
            Param { name: "update_id".to_string(), param_type: "Integer".to_string(), description: "The update\'s unique identifier. Update identifiers start from a certain positive number and increase sequentially. This ID becomes especially handy if you\'re using Webhooks, since it allows you to ignore repeated updates or to restore the correct update sequence, should they get out of order. If there are no new updates for at least a week, then identifier of the next update will be chosen randomly instead of sequentially.".to_string(), required: true },
            Param { name: "message".to_string(), param_type: "Message".to_string(), description: "Optional. New incoming message of any kind — text, photo, sticker, etc.".to_string(), required: false },
            Param { name: "edited_message".to_string(), param_type: "Message".to_string(), description: "Optional. New version of a message that is known to the bot and was edited".to_string(), required: false },
            Param { name: "channel_post".to_string(), param_type: "Message".to_string(), description: "Optional. New incoming channel post of any kind — text, photo, sticker, etc.".to_string(), required: false },
            Param { name: "edited_channel_post".to_string(), param_type: "Message".to_string(), description: "Optional. New version of a channel post that is known to the bot and was edited".to_string(), required: false },
            Param { name: "inline_query".to_string(), param_type: "InlineQuery".to_string(), description: "Optional. New incoming inline query".to_string(), required: false },
            Param { name: "chosen_inline_result".to_string(), param_type: "ChosenInlineResult".to_string(), description: "Optional. The result of an inline query that was chosen by a user and sent to their chat partner. Please see our documentation on the feedback collecting for details on how to enable these updates for your bot.".to_string(), required: false },
            Param { name: "callback_query".to_string(), param_type: "CallbackQuery".to_string(), description: "Optional. New incoming callback query".to_string(), required: false },
            Param { name: "shipping_query".to_string(), param_type: "ShippingQuery".to_string(), description: "Optional. New incoming shipping query. Only for invoices with flexible price".to_string(), required: false },
            Param { name: "pre_checkout_query".to_string(), param_type: "PreCheckoutQuery".to_string(), description: "Optional. New incoming pre-checkout query. Contains full information about checkout".to_string(), required: false },
            Param { name: "poll".to_string(), param_type: "Poll".to_string(), description: "Optional. New poll state. Bots receive only updates about stopped polls and polls, which are sent by the bot".to_string(), required: false },
            Param { name: "poll_answer".to_string(), param_type: "PollAnswer".to_string(), description: "Optional. A user changed their answer in a non-anonymous poll. Bots receive new votes only in polls that were sent by the bot itself.".to_string(), required: false },
            Param { name: "my_chat_member".to_string(), param_type: "ChatMemberUpdated".to_string(), description: "Optional. The bot\'s chat member status was updated in a chat. For private chats, this update is received only when the bot is blocked or unblocked by the user.".to_string(), required: false },
            Param { name: "chat_member".to_string(), param_type: "ChatMemberUpdated".to_string(), description: "Optional. A chat member\'s status was updated in a chat. The bot must be an administrator in the chat and must explicitly specify “chat_member” in the list of allowed_updates to receive these updates.".to_string(), required: false }];

        assert_eq!(expected_params, entity.fields);
    }


    #[test]
    fn it_parses_function_table() {
        let html_table = fs::read_to_string("./test/support/function_table_example.html").unwrap();

        let result = Parser::new(html_table).parse();

        assert_eq!(1, result.functions.len());
        assert_eq!(0, result.entities.len());

        let entity = &result.functions[0];

        assert_eq!("sendMediaGroup".to_string(), entity.name);
        assert_eq!("Use this method to send a group of photos, videos, documents or audios as an album. Documents and audio files can be only grouped in an album with messages of the same type. On success, an array of Messages that were sent is returned.".to_string(), entity.description);

        let expected_params = vec![
            Param { name: "chat_id".to_string(), param_type: "Integer or String".to_string(), description: "Unique identifier for the target chat or username of the target channel (in the format @channelusername)".to_string(), required: true },
            Param { name: "media".to_string(), param_type: "Array of InputMediaAudio, InputMediaDocument, InputMediaPhoto and InputMediaVideo".to_string(), description: "A JSON-serialized array describing messages to be sent, must include 2-10 items".to_string(), required: true },
            Param { name: "disable_notification".to_string(), param_type: "Boolean".to_string(), description: "Sends messages silently. Users will receive a notification with no sound.".to_string(), required: false },
            Param { name: "reply_to_message_id".to_string(), param_type: "Integer".to_string(), description: "If the messages are a reply, ID of the original message".to_string(), required: false },
            Param { name: "allow_sending_without_reply".to_string(), param_type: "Boolean".to_string(), description: "Pass True, if the message should be sent even if the specified replied-to message is not found".to_string(), required: false }
        ];

        assert_eq!(expected_params, entity.params);
    }
}
