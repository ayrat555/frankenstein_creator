use crate::parser::ApiStructure;
use crate::parser::Entity;
use crate::parser::Param;
use codegen::Function;
use codegen::Scope;
use codegen::Struct;

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
        self.generate_entities();

        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.scope.to_string()
    }

    fn generate_entities(&mut self) {
        for entity in &self.structure.entities {
            let strct = self.scope.new_struct(&entity.name).derive("Debug");

            for field in &entity.fields {
                let parsed_type = field.param_type.clone();

                let field_type = if field.required {
                    parsed_type
                } else {
                    format!("Option<{}>", parsed_type)
                };

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

        let expect = r#"#[derive(Debug)]
struct WebhookInfo {
    url: String,
    has_custom_certificate: Boolean,
    pending_update_count: Integer,
    ip_address: Option<String>,
    last_error_date: Option<Integer>,
    last_error_message: Option<String>,
    max_connections: Option<Integer>,
    allowed_updates: Option<Array of String>,
}"#;

        assert!(generator.generate().is_ok());
        assert_eq!(expect, generator.to_string());
    }
}
