use std::collections::HashMap;

pub trait Serde: Sized {
    fn to_json(&self) -> String;
    fn from_json(json: &str) -> Result<Self, String>;
}

impl Serde for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }

    fn from_json(json: &str) -> Result<Self, String> {
        if json.starts_with('"') && json.ends_with('"') {
            Ok(json[1..json.len()-1].to_string())
        } else {
            Err("Invalid JSON string".to_string())
        }
    }
}

impl Serde for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }

    fn from_json(json: &str) -> Result<Self, String> {
        json.parse::<i32>().map_err(|e| e.to_string())
    }
}

impl Serde for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }

    fn from_json(json: &str) -> Result<Self, String> {
        json.parse::<bool>().map_err(|e| e.to_string())
    }
}

impl<T: Serde> Serde for Vec<T> {
    fn to_json(&self) -> String {
        let json_elements: Vec<String> = self.iter().map(|item| item.to_json()).collect();
        format!("[{}]", json_elements.join(", "))
    }

    fn from_json(json: &str) -> Result<Self, String> {
        if json.starts_with('[') && json.ends_with(']') {
            let elements = &json[1..json.len()-1];
            let mut vec = Vec::new();
            for element in elements.split(',') {
                vec.push(T::from_json(element.trim())?);
            }
            Ok(vec)
        } else {
            Err("Invalid JSON array".to_string())
        }
    }
}

impl<K: Serde + Eq + std::hash::Hash, V: Serde> Serde for HashMap<K, V> {
    fn to_json(&self) -> String {
        let json_elements: Vec<String> = self.iter()
            .map(|(key, value)| format!("{}: {}", key.to_json(), value.to_json()))
            .collect();
        format!("{{{}}}", json_elements.join(", "))
    }

    fn from_json(json: &str) -> Result<Self, String> {
        if json.starts_with('{') && json.ends_with('}') {
            let elements = &json[1..json.len()-1];
            let mut map = HashMap::new();
            for element in elements.split(',') {
                let mut kv = element.splitn(2, ':');
                let key = kv.next().ok_or("Missing key")?.trim();
                let value = kv.next().ok_or("Missing value")?.trim();
                map.insert(K::from_json(key)?, V::from_json(value)?);
            }
            Ok(map)
        } else {
            Err("Invalid JSON object".to_string())
        }
    }
}

#[cfg(test)]
#[path = "./json_test.rs"]
mod test;