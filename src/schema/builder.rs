use serde_json::{Map, Value, json};

/// Jason schema builder
/// to be used in place of Schemars
#[derive(Default, Debug, Clone)]
pub struct SchemaBuilder {
    base: Map<String, Value>,
    properties: Map<String, Value>,
    required: Vec<String>,
}

impl SchemaBuilder {
    /// create a new schema builder
    pub fn new() -> Self {
        let mut base = Map::new();
        base.insert("type".to_owned(), json!("object"));

        Self {
            base,
            ..Default::default()
        }
    }

    /// build the json schema as is
    /// should return a valid serde_json::Value
    pub fn build(mut self) -> Value {
        self.base.insert(
            "properties".to_owned(),
            Value::Object(self.properties),
        );

        if !self.required.is_empty() {
            self.base
                .insert("required".to_owned(), json!(self.required));
        }

        Value::Object(self.base)
    }

    /// optional additionalProperties bool value
    /// usually used in providers that require them
    /// to be false, such as openai
    pub fn additional_properties(
        mut self,
        allowed: bool,
    ) -> Self {
        self.base.insert(
            "additionalProperties".to_owned(),
            json!(allowed),
        );

        self
    }

    /// inserts a String object to the schema
    pub fn insert_str(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("string"));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// inserts a bool object to the schema
    pub fn insert_bool(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("boolean"));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// inserts enum values as a string array
    pub fn insert_enum(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        values: &[String],
    ) -> Self {
        let mut prop = Map::new();

        prop.insert("type".to_owned(), json!("string"));
        prop.insert("enum".to_owned(), json!(values));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// insert string array object to the schema
    pub fn insert_str_array(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) -> Self {
        /* "files": {
          "description": "paths to apply commit to\nex. main.rs doubloon.rs",
          "items": {
            "type": "string"
          },
          "type": "array"
        }, */
        let mut items = Map::new();

        items.insert("type".to_owned(), json!("string"));

        let mut prop = Map::new();

        prop.insert("type".to_owned(), json!("array"));
        prop.insert("items".to_owned(), json!(items));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// inserts a enum array with possible values
    /// mainly used to define paths, to choose from
    /// should show as an array
    pub fn insert_enum_array(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        values: &[String],
    ) -> Self {
        let mut items = Map::new();
        items.insert("type".to_owned(), json!("string"));
        items.insert("enum".to_owned(), json!(values));

        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("array"));
        prop.insert("items".to_owned(), Value::Object(items));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }
}
