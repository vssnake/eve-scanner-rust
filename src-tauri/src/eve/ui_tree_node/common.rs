pub mod common {
    #[derive(Debug, Clone)]
    pub struct ColorComponents {
        pub alpha: i32,
        pub red: i32,
        pub green: i32,
        pub blue: i32,
    }


    #[derive(Debug)]
    pub struct Bunch {
        pub entries_of_interest: serde_json::Map<String, serde_json::Value>,
    }
}
