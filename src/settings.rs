pub struct Settings {
    pub openai_api_key: String,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        }
    }
}

impl Clone for Settings {
    fn clone(&self) -> Settings {
        Settings {
            openai_api_key: self.openai_api_key.clone(),
        }
    }
}
