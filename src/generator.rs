use crate::parser::ApiStructure;
use crate::parser::ParsedType;
use crate::parser::RustType;
use codegen::Scope;
use codegen::Variant;

pub struct Generator {
    structure: ApiStructure,
    // result: GeneratedCode,
    scope: Scope,
}

impl Generator {
    pub fn new(structure: ApiStructure) -> Self {
        Self {
            structure,
            scope: Scope::new(),
        }
    }

    pub fn generate(&mut self) -> Result<(), String> {
        self.generate_enums();
        self.generate_entities();
        // self.generate_functions()

        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.scope.to_string()
    }

    fn generate_enums(&mut self) {
        for entity in &self.structure.entities {
            for field in &entity.fields {
                let parsed_type = field.as_rust_type();

                if let RustType::Enum(variants) = parsed_type.rust_type {
                    let enum_name = field.enum_name();
                    let new_enum = self.scope.new_enum(&enum_name);

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

        for function in &self.structure.functions {
            for param in &function.params {
                let parsed_type = param.as_rust_type();

                if let RustType::Enum(variants) = parsed_type.rust_type {
                    let enum_name = param.enum_name();
                    let new_enum = self.scope.new_enum(&enum_name);

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

    fn generate_entities(&mut self) {
        for entity in &self.structure.entities {
            let strct = self.scope.new_struct(&entity.name).derive("Debug");

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

                if type_with_assoc.array {
                    if let RustType::Simple(type_name) = type_with_assoc.rust_type {
                        field_type = format!("Array<{}>", type_name);
                    }
                } else {
                    if let RustType::Simple(type_name) = type_with_assoc.rust_type {
                        field_type = type_name;
                    }
                }

                if type_with_assoc.option {
                    field_type = format!("Option<{}>", field_type)
                }

                strct.field(&field.name, field_type);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use std::fs;

    #[test]
    fn it_creates_struct_from_entity() {
        let html_table =
            fs::read_to_string("./test/support/table_with_entity_and_function_example.html")
                .unwrap();

        let structure = Parser::new(html_table).parse();

        let mut generator = Generator::new(structure);

        let expect = r#"enum ChatIdEnum {
    IsizeVariant(isize),
    StringVariant(String),
}

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
    allowed_updates: Option<Array<String>>,
}"#;

        assert!(generator.generate().is_ok());
        assert_eq!(expect, generator.to_string());
    }
}
