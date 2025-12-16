//! Common types used across Orbis.

use serde::{Deserialize, Serialize};

/// API response wrapper for consistent response format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,

    /// Response data (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error message (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,

    /// Additional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    /// Create an error response.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
            meta: None,
        }
    }

    /// Add metadata to the response.
    #[must_use]
    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// API error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code.
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Pagination parameters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page.
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

const fn default_page() -> u32 {
    1
}

const fn default_per_page() -> u32 {
    20
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl Pagination {
    /// Calculate the offset for database queries.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)).saturating_mul(self.per_page)
    }

    /// Get the limit for database queries.
    #[must_use]
    pub const fn limit(&self) -> u32 {
        self.per_page
    }
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The items for the current page.
    pub items: Vec<T>,

    /// Total number of items.
    pub total: u64,

    /// Current page number.
    pub page: u32,

    /// Items per page.
    pub per_page: u32,

    /// Total number of pages.
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response.
    pub fn new(items: Vec<T>, total: u64, pagination: Pagination) -> Self {
        let total_pages = ((total as f64) / (pagination.per_page as f64)).ceil() as u32;
        Self {
            items,
            total,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages,
        }
    }
}
