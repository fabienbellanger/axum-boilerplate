//! Query module is used for paginage, sorting and filtering API (TODO: separate)

// https://www.moesif.com/blog/technical/api-design/REST-API-Design-Filtering-Sorting-and-Pagination/

// TODO: Add:
// - sort (Ex.: ?sort=+lastname,-firstname) {+: ASC, -: DESC}
// - filter (Ex.: ?lastname=eq:toto&first=ne:tutu&age=lt:18,gt:5) => {eq, ne, gt, ge, lt, le}

use serde::Deserialize;

const PAGINATION_MAX_LIMIT: usize = 500;

/// Query parameters used to paginate API
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PaginateSortQuery {
    #[serde(rename(deserialize = "p"))]
    pub page: Option<usize>,

    #[serde(rename(deserialize = "l"))]
    pub limit: Option<usize>,

    #[serde(rename(deserialize = "s"))]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum Sort {
    /// Ascending sort (`'+'` prefix)
    Asc,

    /// Descending sort (`'-'` prefix)
    Desc,
}

/// Parameters used to paginate and/or sort database results
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PaginateSort {
    pub page: usize,
    pub limit: usize,
    pub offset: usize,
    pub sorts: Vec<(String, Sort)>,
}

impl From<PaginateSortQuery> for PaginateSort {
    fn from(value: PaginateSortQuery) -> Self {
        // Page
        let page = match value.page {
            Some(page) => match page >= 1 {
                true => page,
                false => 1,
            },
            None => 1,
        };

        // Limit
        let limit = match value.limit {
            Some(limit) => {
                if !(1..=PAGINATION_MAX_LIMIT).contains(&limit) {
                    PAGINATION_MAX_LIMIT
                } else {
                    limit
                }
            }
            None => PAGINATION_MAX_LIMIT,
        };

        // Offset
        let offset = (page - 1) * limit;

        // Sort
        let mut sorts = vec![];
        let sort = value.sort.unwrap_or_default();
        let sort_parts = sort.split(',');
        for part in sort_parts {
            let prefix = part.chars().next();
            if let Some(prefix) = prefix {
                if prefix == '+' {
                    sorts.push((part[1..].to_string(), Sort::Asc));
                } else if prefix == '-' {
                    sorts.push((part[1..].to_string(), Sort::Desc));
                }
            }
        }

        Self {
            page,
            limit,
            offset,
            sorts,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_paginate_sort_qurey_paginate() {
        let data = PaginateSortQuery {
            page: None,
            limit: None,
            sort: None,
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![],
            },
            data
        );

        let data = PaginateSortQuery {
            page: None,
            limit: Some(600),
            sort: None,
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![],
            },
            data
        );

        let data = PaginateSortQuery {
            page: Some(0),
            limit: None,
            sort: None,
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![],
            },
            data
        );

        let data = PaginateSortQuery {
            page: Some(2),
            limit: Some(100),
            sort: None,
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 2,
                limit: 100,
                offset: 100,
                sorts: vec![],
            },
            data
        );
    }

    #[test]
    fn test_from_paginate_sort_qurey_sort() {
        let data = PaginateSortQuery {
            page: None,
            limit: None,
            sort: None,
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![],
            },
            data
        );

        let data = PaginateSortQuery {
            page: None,
            limit: None,
            sort: Some("+id,-created_at".to_owned()),
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![("id".to_owned(), Sort::Asc), ("created_at".to_owned(), Sort::Desc)],
            },
            data
        );

        let data = PaginateSortQuery {
            page: None,
            limit: None,
            sort: Some("created_at".to_owned()),
        };
        let data: PaginateSort = data.into();
        assert_eq!(
            PaginateSort {
                page: 1,
                limit: PAGINATION_MAX_LIMIT,
                offset: 0,
                sorts: vec![],
            },
            data
        );
    }
}
