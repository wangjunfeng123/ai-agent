use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::agent::Event;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: Option<String>,
    pub events: Vec<Event>,
    pub state: HashMap<String, Value>,
    pub create_at: DateTime<Utc>,
    pub update_at: DateTime<Utc>,
}

impl Session {
    pub fn new(session_id: String, user_id: Option<String>) -> Self {
        Self {
            session_id,
            user_id,
            events: Vec::new(),
            state: HashMap::new(),
            create_at: Utc::now(),
            update_at: Utc::now(),
        }
    }
}
