pub mod common {
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize)]
    pub struct ColorComponents {
        pub alpha: i32,
        pub red: i32,
        pub green: i32,
        pub blue: i32,
    }


    #[derive(Debug, Serialize)]
    pub struct Bunch {
        pub entries_of_interest: serde_json::Map<String, serde_json::Value>,
    }
}
