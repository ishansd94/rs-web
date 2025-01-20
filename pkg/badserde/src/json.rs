use std::collections::HashMap;

pub trait ToJson {
    fn to_json(&self) -> String;
}

pub trait FromJson: Sized {
    fn from_json(json: &str) -> Result<Self, String>;
}

impl ToJson for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl FromJson for String {
    fn from_json(json: &str) -> Result<Self, String> {
        if json.starts_with('"') && json.ends_with('"') {
            Ok(json[1..json.len()-1].to_string())
        } else {
            Err("Invalid JSON string".to_string())
        }
    }
}

impl ToJson for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl FromJson for i32 {
    fn from_json(json: &str) -> Result<Self, String> {
        json.parse::<i32>().map_err(|e| e.to_string())
    }
}

impl ToJson for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl FromJson for bool {
    fn from_json(json: &str) -> Result<Self, String> {
        json.parse::<bool>().map_err(|e| e.to_string())
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> String {
        let json_elements: Vec<String> = self.iter().map(|item| item.to_json()).collect();
        format!("[{}]", json_elements.join(", "))
    }
}

impl<T: FromJson> FromJson for Vec<T> {
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

impl<K: ToJson, V: ToJson> ToJson for HashMap<K, V> {
    fn to_json(&self) -> String {
        let json_elements: Vec<String> = self.iter()
            .map(|(key, value)| format!("{}: {}", key.to_json(), value.to_json()))
            .collect();
        format!("{{{}}}", json_elements.join(", "))
    }
}

impl<K: FromJson + Eq + std::hash::Hash, V: FromJson> FromJson for HashMap<K, V> {
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
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_string_to_json() {
        let my_string = "Hello, world!".to_string();
        assert_eq!(my_string.to_json(), "\"Hello, world!\"");
    }

    #[test]
    fn test_string_from_json() {
        let json_str = "\"Hello, world!\"";
        let my_string: String = FromJson::from_json(json_str).unwrap();
        assert_eq!(my_string, "Hello, world!");
    }

    #[test]
    fn test_i32_to_json() {
        let my_int = 42;
        assert_eq!(my_int.to_json(), "42");
    }

    #[test]
    fn test_i32_from_json() {
        let json_int = "42";
        let my_int: i32 = FromJson::from_json(json_int).unwrap();
        assert_eq!(my_int, 42);
    }

    #[test]
    fn test_bool_to_json() {
        let my_bool = true;
        assert_eq!(my_bool.to_json(), "true");
        let my_bool = false;
        assert_eq!(my_bool.to_json(), "false");
    }

    #[test]
    fn test_bool_from_json() {
        let json_bool = "true";
        let my_bool: bool = FromJson::from_json(json_bool).unwrap();
        assert_eq!(my_bool, true);
        let json_bool = "false";
        let my_bool: bool = FromJson::from_json(json_bool).unwrap();
        assert_eq!(my_bool, false);
    }

    #[test]
    fn test_vec_to_json() {
        let my_vec = vec![1, 2, 3];
        assert_eq!(my_vec.to_json(), "[1, 2, 3]");
    }

    #[test]
    fn test_vec_from_json() {
        let json_vec = "[1, 2, 3]";
        let my_vec: Vec<i32> = FromJson::from_json(json_vec).unwrap();
        assert_eq!(my_vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_hashmap_to_json() {
        let mut my_map = HashMap::new();
        my_map.insert("key1".to_string(), "value1".to_string());
        let json = my_map.to_json();
        assert_eq!(json, "{\"key1\": \"value1\"}");
    }

    #[test]
    fn test_hashmap_from_json() {
        let json_map = "{\"key1\": \"value1\", \"key2\": \"value2\"}";
        let my_map: HashMap<String, String> = FromJson::from_json(json_map).unwrap();
        assert_eq!(my_map.get("key1").unwrap(), "value1");
        assert_eq!(my_map.get("key2").unwrap(), "value2");
    }
}