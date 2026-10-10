use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PER_PAGE: u64 = 20;
const MAX_PER_PAGE: u64 = 100;

/// Bounds violations for pagination query parameters.
///
/// Typed so callers cannot rely on error-string matching and so the
/// transport layer maps an explicit variant to a stable error code.
#[derive(Debug, Clone, Copy, Error, Eq, PartialEq)]
pub enum PaginationError {
    #[error("page must be at least 1")]
    PageTooSmall,
    #[error("page exceeds the supported offset range")]
    PageTooLarge,
    #[error("per_page must be between 1 and {max}")]
    PerPageOutOfRange { max: u64 },
}

/// Query convention for offset pagination under `/v1`.
///
/// `?page=1&per_page=20` — both optional, validated, never 500.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl PaginationParams {
    /// Validate bounds. Returns a typed error the transport maps to 400.
    pub fn validate(self) -> Result<Self, PaginationError> {
        if self.page == 0 {
            return Err(PaginationError::PageTooSmall);
        }
        if self.per_page == 0 || self.per_page > MAX_PER_PAGE {
            return Err(PaginationError::PerPageOutOfRange { max: MAX_PER_PAGE });
        }
        if (self.page - 1)
            .checked_mul(self.per_page)
            .is_none_or(|offset| offset > i64::MAX as u64)
        {
            return Err(PaginationError::PageTooLarge);
        }
        Ok(self)
    }

    #[must_use]
    pub const fn offset(self) -> u64 {
        self.page.saturating_sub(1).saturating_mul(self.per_page)
    }

    #[must_use]
    pub const fn limit(self) -> u64 {
        self.per_page
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Paginated<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T: Serialize> Paginated<T> {
    #[must_use]
    pub fn new(data: Vec<T>, params: PaginationParams, total: Option<u64>) -> Self {
        Self {
            data,
            pagination: PaginationMeta {
                page: params.page,
                per_page: params.per_page,
                total,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let p = PaginationParams::default()
            .validate()
            .expect("defaults valid");
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 20);
    }

    #[test]
    fn rejects_zero_page_with_typed_error() {
        let err = PaginationParams {
            page: 0,
            per_page: 20,
        }
        .validate()
        .expect_err("must reject");
        assert_eq!(err, PaginationError::PageTooSmall);
    }

    #[test]
    fn rejects_over_max_per_page_with_typed_error() {
        let err = PaginationParams {
            page: 1,
            per_page: 101,
        }
        .validate()
        .expect_err("must reject");
        assert_eq!(
            err,
            PaginationError::PerPageOutOfRange { max: MAX_PER_PAGE }
        );
    }

    #[test]
    fn rejects_zero_per_page_with_typed_error() {
        let err = PaginationParams {
            page: 1,
            per_page: 0,
        }
        .validate()
        .expect_err("must reject");
        assert_eq!(
            err,
            PaginationError::PerPageOutOfRange { max: MAX_PER_PAGE }
        );
    }

    #[test]
    fn rejects_offsets_outside_postgres_range() {
        for page in [u64::MAX, (i64::MAX as u64 / 100) + 2] {
            assert_eq!(
                PaginationParams {
                    page,
                    per_page: 100
                }
                .validate()
                .expect_err("invalid offset"),
                PaginationError::PageTooLarge
            );
        }
    }

    #[test]
    fn accepts_largest_supported_offset() {
        let params = PaginationParams {
            page: i64::MAX as u64 + 1,
            per_page: 1,
        }
        .validate()
        .expect("representable offset");
        assert_eq!(params.offset(), i64::MAX as u64);
    }

    #[test]
    fn offset_math() {
        let p = PaginationParams {
            page: 3,
            per_page: 20,
        }
        .validate()
        .expect("ok");
        assert_eq!(p.offset(), 40);
        assert_eq!(p.limit(), 20);
    }
}
