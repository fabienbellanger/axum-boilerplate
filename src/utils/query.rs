//! Query module is used for paginage, sorting and filtering API

// https://www.moesif.com/blog/technical/api-design/REST-API-Design-Filtering-Sorting-and-Pagination/

// TODO: Add:
// - sort (Ex.: ?sort=+lastname,-firstname) {+: ASC, -: DESC}
// - filter (Ex.: ?lastname=eq:toto&first=ne:tutu&age=lt:18,gt:5) => {eq, ne, gt, ge, lt, le}

use serde::Deserialize;

const PAGINATION_MAX_LIMIT: usize = 500;

/// Query parameters used to paginate API
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PaginateQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl PaginateQuery {
    /// Check and update with correct values
    pub fn build(&mut self) {
        // Page
        if let Some(page) = self.page {
            if page < 1 {
                self.page = Some(1);
            }
        } else {
            self.page = Some(1);
        }

        // Limit
        if let Some(limit) = self.limit {
            if !(1..=PAGINATION_MAX_LIMIT).contains(&limit) {
                self.limit = Some(PAGINATION_MAX_LIMIT);
            }
        } else {
            self.limit = Some(PAGINATION_MAX_LIMIT);
        }

        // Offset
        self.offset = match (self.page, self.limit) {
            (Some(page), Some(limit)) => Some((page - 1) * limit),
            _ => Some(0),
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_paginate() {
        let mut data = PaginateQuery {
            page: None,
            limit: None,
            offset: None,
        };
        data.build();
        assert_eq!(
            PaginateQuery {
                page: Some(1),
                limit: Some(PAGINATION_MAX_LIMIT),
                offset: Some(0),
            },
            data
        );

        let mut data = PaginateQuery {
            page: None,
            limit: Some(600),
            offset: None,
        };
        data.build();
        assert_eq!(
            PaginateQuery {
                page: Some(1),
                limit: Some(PAGINATION_MAX_LIMIT),
                offset: Some(0),
            },
            data
        );

        let mut data = PaginateQuery {
            page: Some(0),
            limit: None,
            offset: None,
        };
        data.build();
        assert_eq!(
            PaginateQuery {
                page: Some(1),
                limit: Some(PAGINATION_MAX_LIMIT),
                offset: Some(0),
            },
            data
        );

        let mut data = PaginateQuery {
            page: Some(2),
            limit: Some(100),
            offset: None,
        };
        data.build();
        assert_eq!(
            PaginateQuery {
                page: Some(2),
                limit: Some(100),
                offset: Some(100),
            },
            data
        );
    }
}
