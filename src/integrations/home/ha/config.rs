use std::collections::HashMap;

pub fn ha_url(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("HOME_ASSISTANT_ENDPOINT").map(String::as_str)
}

pub fn ha_token(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("HOME_ASSISTANT_TOKEN").map(String::as_str)
}

