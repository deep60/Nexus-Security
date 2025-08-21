use serde::{Deserialize, Serialize};

pub mod common {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub success: bool,
        pub data: Option<T>,
        pub message: Option<String>,
        pub error: Option<String>,
    }

    impl<T> ApiResponse<T> {
        pub fn success(data: T, message: Option<String>) -> Self {
            Self {
                success: true,
                data: Some(data),
                message,
                error: None,
            }
        }

        pub fn success_no_data(message: String) -> Self {
            Self {
                success: true,
                data: None,
                message: Some(message),
                error: None,
            }
        }

        pub fn error(error: String) -> Self {
            Self {
                success: false,
                data: None,
                message: None,
                error: Some(error),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PaginationParams {
        pub page: Option<u32>,
        pub per_page: Option<u32>,
        pub sort_by: Option<String>,
        pub sort_order: Option<String>,
    }

    impl Default for PaginationParams {
        fn default() -> Self {
            Self {
                page: Some(1),
                per_page: Some(20),
                sort_by: None,
                sort_order: Some("asc".to_string()),
            }
        }
    }
}
