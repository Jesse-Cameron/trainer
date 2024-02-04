use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Minutes {
    pub minutes: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Departures {
    pub to_city_departures: Vec<Minutes>,
    pub from_city_departures: Vec<Minutes>,
}
