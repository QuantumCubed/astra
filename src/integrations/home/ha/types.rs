use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HaArea {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct HaDevice {
    pub entity_id: String,
    pub friendly_name: String,
    pub aliases: Vec<String>,
    pub area: Option<HaArea>,
    pub state: String,
}
