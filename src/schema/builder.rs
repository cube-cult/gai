use serde_json::{Map, Value, json};

/// lightweight schema settings
#[derive(Default, Debug, Clone)]
pub struct SchemaSettings {
    pub additional_properties: Option<bool>,
}

/// Jason schema builder
/// to be used in place of Schemars
/// based on openai's supported schema properties
/// https://platform.openai.com/docs/guides/structured-outputs#supported-schemas
#[derive(Default, Debug, Clone)]
pub struct SchemaBuilder {
    base: Map<String, Value>,
    properties: Map<String, Value>,
    required: Vec<String>,

    settings: SchemaSettings,
}

impl SchemaSettings {
    /// optional additionalProperties bool value
    /// usually used in providers that require them
    /// to be false, such as openai
    pub fn additional_properties(
        mut self,
        value: bool,
    ) -> Self {
        self.additional_properties = Some(value);
        self
    }
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

        if let Some(v) = self.settings.additional_properties {
            self.base
                .insert("additionalProperties".to_owned(), json!(v));
        }

        if !self.required.is_empty() {
            self.base
                .insert("required".to_owned(), json!(self.required));
        }

        Value::Object(self.base)
    }

    /// build the schema as a raw Value for
    /// nestin, it does not include
    /// the outer wrapper
    pub fn build_inner(self) -> Value {
        let mut obj = Map::new();
        obj.insert("type".to_owned(), json!("object"));
        obj.insert(
            "properties".to_owned(),
            Value::Object(self.properties),
        );

        if let Some(v) = self.settings.additional_properties {
            obj.insert("additionalProperties".to_owned(), json!(v));
        }

        if !self.required.is_empty() {
            obj.insert("required".to_owned(), json!(self.required));
        }

        Value::Object(obj)
    }

    /// set schema builder settings
    pub fn settings(
        mut self,
        settings: SchemaSettings,
    ) -> Self {
        self.settings = settings.to_owned();
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

    /// inserts an array of objects
    /// using a nested SchemaBuilder
    /// useful for creating arrays of
    /// more complex objects like commits
    pub fn insert_object_array(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        item_schema: Value,
    ) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("array"));
        prop.insert("items".to_owned(), item_schema);

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// inserts a nested object using with another
    /// SchemaBuilder
    pub fn insert_object(
        mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        nested_schema: Value,
    ) -> Self {
        let mut prop = match nested_schema {
            Value::Object(map) => map,
            _ => Map::new(),
        };

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }

        self
    }

    /// add a str property
    pub fn add_str(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("string"));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }
    }

    /// add a bool property
    pub fn add_bool(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("boolean"));

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }
    }

    /// add an enum property
    pub fn add_enum(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        values: &[String],
    ) {
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
    }

    /// add a string array property
    pub fn add_str_array(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
    ) {
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
    }

    /// oadd an enum array property
    pub fn add_enum_array(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        values: &[String],
    ) {
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
    }

    /// add an object array property
    pub fn add_object_array(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        item_schema: Value,
    ) {
        let mut prop = Map::new();
        prop.insert("type".to_owned(), json!("array"));
        prop.insert("items".to_owned(), item_schema);

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }
    }

    /// add a nested object property
    pub fn add_object(
        &mut self,
        name: &str,
        description: Option<&str>,
        required: bool,
        nested_schema: Value,
    ) {
        let mut prop = match nested_schema {
            Value::Object(map) => map,
            _ => Map::new(),
        };

        if let Some(description) = description {
            prop.insert("description".to_owned(), json!(description));
        }

        self.properties.insert(name.to_owned(), Value::Object(prop));

        if required {
            self.required.push(name.to_owned());
        }
    }
}
