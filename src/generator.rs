use crate::parser::ApiStructure;
use crate::parser::ParsedType;
use crate::parser::RustType;
use codegen::Field;
use codegen::Scope;
use codegen::Type;
use codegen::Variant;
use heck::CamelCase;

pub struct Generator {
    structure: ApiStructure,
    created_enums: Vec<String>,
    created_structs: Vec<(String, Vec<(String, String)>, Vec<(String, String)>)>,
    scope: Scope,
}

impl Generator {
    pub fn new(structure: ApiStructure) -> Self {
        Self {
            structure,
            scope: Scope::new(),
            created_enums: vec![],
            created_structs: vec![],
        }
    }

    pub fn generate(&mut self) {
        self.generate_enums();
        self.generate_structs();
        self.generate_functions();
    }

    pub fn generate_function_data(&mut self) {
        self.generate_function_enums();
        self.generate_function_structs();
        self.generate_functions();
    }

    pub fn generate_entity_data(&mut self) {
        self.generate_entity_enums();
        self.generate_entity_structs();
        self.generate_functions();
    }

    pub fn to_string(&self) -> String {
        self.scope.to_string()
    }

    fn generate_enums(&mut self) {
        self.generate_entity_enums();
        self.generate_function_enums();
    }

    fn generate_structs(&mut self) {
        self.generate_entity_structs();
        self.generate_function_structs();
    }

    fn generate_functions(&mut self) {
        for (struct_name, required_fields, optional_fields) in &self.created_structs {
            let imp = self.scope.new_impl(struct_name);

            let new_fn = imp.new_fn("new").vis("pub").ret(Type::new("Self"));

            let mut body = "Self {".to_string();

            for (required_field_name, required_field_type) in required_fields {
                new_fn.arg(required_field_name, Type::new(required_field_type));

                body.push_str(&format!("{},", required_field_name));
            }

            for (optional_field_name, _) in optional_fields {
                body.push_str(&format!("{}: None,", optional_field_name));
            }

            body.push_str("}");

            new_fn.line(body);

            for (required_field_name, required_field_type) in required_fields {
                imp.new_fn(&format!("set_{}", required_field_name))
                    .vis("pub")
                    .arg_mut_self()
                    .arg(required_field_name, Type::new(required_field_type))
                    .line(&format!(
                        "self.{} = {};",
                        required_field_name, required_field_name
                    ));
            }

            for (optional_field_name, optional_field_type) in optional_fields {
                imp.new_fn(&format!("set_{}", optional_field_name))
                    .vis("pub")
                    .arg_mut_self()
                    .arg(
                        optional_field_name,
                        Type::new(&format!("Option<{}>", optional_field_type)),
                    )
                    .line(&format!(
                        "self.{} = {};",
                        optional_field_name, optional_field_name
                    ));
            }

            for (required_field_name, required_field_type) in required_fields {
                let body = match required_field_type.as_str() {
                    "isize" | "f64" | "bool" => format!("self.{}", required_field_name),
                    _ => format!("self.{}.clone()", required_field_name),
                };

                imp.new_fn(&format!("{}", required_field_name))
                    .vis("pub")
                    .arg_ref_self()
                    .line(&body)
                    .ret(Type::new(required_field_type));
            }

            for (optional_field_name, optional_field_type) in optional_fields {
                imp.new_fn(&format!("{}", optional_field_name))
                    .vis("pub")
                    .arg_ref_self()
                    .line(&format!("self.{}.clone()", optional_field_name))
                    .ret(Type::new(&format!("Option<{}>", optional_field_type)));
            }
        }
    }

    fn generate_entity_enums(&mut self) {
        for entity in &self.structure.entities {
            for field in &entity.fields {
                let parsed_type = field.as_rust_type();

                if let RustType::Enum(variants) = parsed_type.rust_type {
                    let enum_name = field.enum_name();

                    if !self.created_enums.contains(&enum_name) {
                        self.created_enums.push(enum_name.clone());

                        let new_enum = self
                            .scope
                            .new_enum(&enum_name)
                            .vis("pub")
                            .derive("Clone")
                            .derive("Debug")
                            .derive("Serialize")
                            .derive("Deserialize")
                            .derive("PartialEq");

                        for rust_type in variants {
                            match rust_type {
                                RustType::Simple(_) => {
                                    new_enum.push_variant(Variant::new(&rust_type.variant_name()));
                                }

                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_function_enums(&mut self) {
        for function in &self.structure.functions {
            for param in &function.params {
                let parsed_type = param.as_rust_type();

                if let RustType::Enum(variants) = parsed_type.rust_type {
                    let enum_name = param.enum_name();

                    if !self.created_enums.contains(&enum_name) {
                        self.created_enums.push(enum_name.clone());

                        let new_enum = self
                            .scope
                            .new_enum(&enum_name)
                            .vis("pub")
                            .derive("Clone")
                            .derive("Debug")
                            .derive("Serialize")
                            .derive("Deserialize")
                            .derive("PartialEq");

                        for rust_type in variants {
                            match rust_type {
                                RustType::Simple(_) => {
                                    new_enum.push_variant(Variant::new(&rust_type.variant_name()));
                                }

                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_entity_structs(&mut self) {
        for entity in &self.structure.entities {
            let strct = self
                .scope
                .new_struct(&entity.name)
                .vis("pub")
                .derive("Debug")
                .derive("Clone")
                .derive("Serialize")
                .derive("Deserialize")
                .derive("PartialEq");

            let mut required_fields: Vec<(String, String)> = vec![];
            let mut optional_fields: Vec<(String, String)> = vec![];

            for field in &entity.fields {
                let parsed_type = field.as_rust_type();

                let type_with_assoc = match parsed_type.rust_type {
                    RustType::Simple(_) => parsed_type,
                    RustType::Enum(_) => {
                        let enum_name = field.enum_name();

                        ParsedType {
                            array: parsed_type.array,
                            option: parsed_type.option,
                            rust_type: RustType::Simple(enum_name),
                        }
                    }
                };

                let mut field_type: String = "".to_string();

                if let RustType::Simple(type_name) = type_with_assoc.rust_type {
                    if type_name == entity.name {
                        field_type = format!("Box<{}>", type_name);
                    } else {
                        field_type = type_name;
                    }
                }

                if type_with_assoc.array {
                    field_type = format!("Vec<{}>", field_type);
                }

                if type_with_assoc.option {
                    optional_fields.push((field.field_name(), field_type.clone()));
                    field_type = format!("Option<{}>", field_type)
                } else {
                    required_fields.push((field.field_name(), field_type.clone()));
                }

                let mut gen_field = Field::new(&field.field_name(), field_type);

                gen_field.annotation(vec![&field.annotation()]);

                strct.push_field(gen_field);
            }

            self.created_structs
                .push((entity.name.clone(), required_fields, optional_fields));
        }
    }

    fn generate_function_structs(&mut self) {
        for function in &self.structure.functions {
            let struct_name = format!("{}Params", function.name.to_camel_case());
            let strct = self
                .scope
                .new_struct(&struct_name)
                .vis("pub")
                .derive("Debug")
                .derive("Clone")
                .derive("Serialize")
                .derive("Deserialize")
                .derive("PartialEq");

            let mut required_fields: Vec<(String, String)> = vec![];
            let mut optional_fields: Vec<(String, String)> = vec![];

            for field in &function.params {
                let parsed_type = field.as_rust_type();

                let type_with_assoc = match parsed_type.rust_type {
                    RustType::Simple(_) => parsed_type,
                    RustType::Enum(_) => {
                        let enum_name = field.enum_name();

                        ParsedType {
                            array: parsed_type.array,
                            option: parsed_type.option,
                            rust_type: RustType::Simple(enum_name),
                        }
                    }
                };

                let mut field_type: String = "".to_string();

                if type_with_assoc.array {
                    if let RustType::Simple(type_name) = type_with_assoc.rust_type {
                        field_type = format!("Vec<{}>", type_name);
                    }
                } else {
                    if let RustType::Simple(type_name) = type_with_assoc.rust_type {
                        field_type = type_name;
                    }
                }

                if type_with_assoc.option {
                    optional_fields.push((field.field_name(), field_type.clone()));
                    field_type = format!("Option<{}>", field_type)
                } else {
                    required_fields.push((field.field_name(), field_type.clone()));
                }

                let mut gen_field = Field::new(&field.field_name(), field_type);

                gen_field.annotation(vec![&field.annotation()]);

                strct.push_field(gen_field);
            }
            self.created_structs
                .push((struct_name, required_fields, optional_fields));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use std::fs;

    #[test]
    fn it_creates_structs() {
        let html_table =
            fs::read_to_string("./test/support/table_with_entity_and_function_example.html")
                .unwrap();

        let structure = Parser::new(html_table).parse();

        let mut generator = Generator::new(structure);

        let expect = r#"#[derive(Debug)]
enum ChatIdEnum {
    IsizeVariant(isize),
    StringVariant(String),
}

#[derive(Debug)]
enum FromChatIdEnum {
    IsizeVariant(isize),
    StringVariant(String),
}

#[derive(Debug)]
struct WebhookInfo {
    url: String,
    has_custom_certificate: bool,
    pending_update_count: isize,
    ip_address: Option<String>,
    last_error_date: Option<isize>,
    last_error_message: Option<String>,
    max_connections: Option<isize>,
    allowed_updates: Option<Vec<String>>,
}

#[derive(Debug)]
struct ForwardMessageParams {
    chat_id: ChatIdEnum,
    from_chat_id: FromChatIdEnum,
    disable_notification: Option<bool>,
    message_id: isize,
}"#;

        generator.generate();

        assert_eq!(expect, generator.to_string());
    }
}
