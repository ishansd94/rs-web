use std::collections::HashMap;

pub trait ToJson {
    fn to_json(&self) -> String;
}

impl ToJson for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl ToJson for &str {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl ToJson for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl ToJson for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> String {
        let json_elements: Vec<String> = self.iter().map(|item| item.to_json()).collect();
        format!("[{}]", json_elements.join(", "))
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
    fn test_i32_to_json() {
        let my_int = 42;
        assert_eq!(my_int.to_json(), "42");
    }

    #[test]
    fn test_bool_to_json() {
        let my_bool = true;
        assert_eq!(my_bool.to_json(), "true");
        let my_bool = false;
        assert_eq!(my_bool.to_json(), "false");
    }

    #[test]
    fn test_vec_to_json() {
        let my_vec = vec![1, 2, 3];
        assert_eq!(my_vec.to_json(), "[1, 2, 3]");
    }

    #[test]
    fn test_hashmap_to_json() {
        let mut my_map = HashMap::new();
        my_map.insert("key1".to_string(), "value1".to_string());
        my_map.insert("key2".to_string(), "value2".to_string());
        let json = my_map.to_json();
        assert!(json.contains("\"key1\": \"value1\""));
        assert!(json.contains("\"key2\": \"value2\""));
    }
}