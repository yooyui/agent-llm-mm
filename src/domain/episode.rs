use crate::domain::event::Event;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Episode {
    pub title: String,
    pub events: Vec<Event>,
}

impl Episode {
    pub fn new(title: impl Into<String>, events: Vec<Event>) -> Self {
        Self {
            title: title.into(),
            events,
        }
    }
}
