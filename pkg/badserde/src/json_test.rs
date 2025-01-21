#[cfg(test)]
mod tests {
    use super::super::Serde;
    use std::collections::HashMap;

    #[test]
    fn test_string_to_json() {
        let my_string = "Hello, world!".to_string();
        assert_eq!(my_string.to_json(), "\"Hello, world!\"");
    }

    #[test]
    fn test_string_from_json() {
        let json_str = "\"Hello, world!\"";
        let my_string: String = Serde::from_json(json_str).unwrap();
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
        let my_int: i32 = Serde::from_json(json_int).unwrap();
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
        let my_bool: bool = Serde::from_json(json_bool).unwrap();
        assert_eq!(my_bool, true);
        let json_bool = "false";
        let my_bool: bool = Serde::from_json(json_bool).unwrap();
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
        let my_vec: Vec<i32> = Serde::from_json(json_vec).unwrap();
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
        let my_map: HashMap<String, String> = Serde::from_json(json_map).unwrap();
        assert_eq!(my_map.get("key1").unwrap(), "value1");
        assert_eq!(my_map.get("key2").unwrap(), "value2");
    }
}