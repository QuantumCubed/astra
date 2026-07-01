use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HaDevice {
    pub entity_id: String,
    pub friendly_name: String,
    pub aliases: Vec<String>,
    pub area: Option<String>,
    pub state: String,
}
