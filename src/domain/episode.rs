use crate::domain::event::Event;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Episode {
    title: String,
    events: Vec<Event>,
}

impl Episode {
    pub fn new(title: impl Into<String>, events: Vec<Event>) -> Self {
        Self {
            title: title.into(),
            events,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }
}
