//! Query module is used for paginage, sorting and filtering API (TODO: separate)

// https://www.moesif.com/blog/technical/api-design/REST-API-Design-Filtering-Sorting-and-Pagination/

use serde::{Deserialize, Serialize};
use std::fmt::Display;

const PAGINATION_MAX_LIMIT: u32 = 500;

#[derive(Serialize)]
pub struct PaginateResponse<T: Serialize> {
    pub data: T,
    pub total: i64,
}

/// Query parameters used to paginate API
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PaginateSortQuery {
    #[serde(rename(deserialize = "p"))]
    pub page: Option<u32>,

    #[serde(rename(deserialize = "l"))]
    pub limit: Option<u32>,

    #[serde(rename(deserialize = "s"))]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum Sort {
    /// Ascending sort (`'+'` prefix)
    /// Example: ?sort=+id
    Asc,

    /// Descending sort (`'-'` prefix)
    /// Example: ?sort=-name
    Desc,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Asc => "ASC",
                Self::Desc => "DESC",
            }
        )
    }
}

/// Parameters used to paginate and/or sort database results
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PaginateSort {
    pub page: u32,
    pub limit: u32,
    pub offset: u32,
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

impl PaginateSort {
    /// SQL code for pagination
    pub fn get_pagination_sql(&self) -> String {
        format!(" LIMIT {} OFFSET {}", self.limit, self.offset)
    }

    /// SQL code for sorts (ORDER BY)
    // TODO: Does not manage tables name!
    pub fn get_sorts_sql(&self, valid_fields: Option<&[&str]>) -> String {
        let mut s = String::new();
        let mut i = 0;

        for (field, sort) in self.sorts.iter() {
            match &valid_fields {
                Some(valid_fields) => {
                    if valid_fields.contains(&field.as_str()) {
                        if i == 0 {
                            s.push_str(" ORDER BY ");
                        } else {
                            s.push_str(", ");
                        }
                        s.push_str(&format!("{field} {sort}"));

                        i += 1;
                    }
                }
                None => {
                    if i == 0 {
                        s.push_str(" ORDER BY ");
                    } else {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("{field} {sort}"));

                    i += 1;
                }
            }
        }

        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_paginate_sort_query_paginate() {
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
    fn test_from_paginate_sort_query_sort() {
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

    #[test]
    fn test_get_pagination_sql() {
        let paginate_sort = PaginateSort {
            page: 1,
            limit: 50,
            offset: 0,
            sorts: vec![],
        };
        assert_eq!(String::from(" LIMIT 50 OFFSET 0"), paginate_sort.get_pagination_sql());
    }

    #[test]
    fn test_get_sorts_sql_without_sort() {
        let paginate_sort = PaginateSort {
            page: 1,
            limit: 50,
            offset: 0,
            sorts: vec![],
        };
        assert_eq!(String::new(), paginate_sort.get_sorts_sql(None));
    }

    #[test]
    fn test_get_sorts_sql_without_valid_fields() {
        let mut valid_fields: Option<&[&str]> = Some(&[]);

        let mut paginate_sort = PaginateSort {
            page: 1,
            limit: 50,
            offset: 0,
            sorts: vec![],
        };
        assert_eq!(String::new(), paginate_sort.get_sorts_sql(valid_fields));

        paginate_sort.sorts = vec![("id".to_owned(), Sort::Asc), ("name".to_owned(), Sort::Desc)];
        assert_eq!(String::new(), paginate_sort.get_sorts_sql(valid_fields));

        valid_fields = None;
        assert_eq!(
            String::from(" ORDER BY id ASC, name DESC"),
            paginate_sort.get_sorts_sql(valid_fields)
        );
    }

    #[test]
    fn test_get_sorts_sql_with_valid_fields() {
        let valid_fields: Option<&[&str]> = Some(&["id", "name"]);

        let mut paginate_sort = PaginateSort {
            page: 1,
            limit: 50,
            offset: 0,
            sorts: vec![],
        };
        assert_eq!(String::new(), paginate_sort.get_sorts_sql(valid_fields));

        paginate_sort.sorts = vec![("id".to_owned(), Sort::Asc), ("name".to_owned(), Sort::Desc)];
        assert_eq!(
            " ORDER BY id ASC, name DESC".to_owned(),
            paginate_sort.get_sorts_sql(valid_fields)
        );

        let valid_fields: Option<&[&str]> = Some(&["name"]);
        assert_eq!(
            " ORDER BY name DESC".to_owned(),
            paginate_sort.get_sorts_sql(valid_fields)
        );

        let valid_fields: Option<&[&str]> = Some(&["id", "name"]);
        paginate_sort.sorts = vec![("idz".to_owned(), Sort::Asc), ("name".to_owned(), Sort::Desc)];
        assert_eq!(
            " ORDER BY name DESC".to_owned(),
            paginate_sort.get_sorts_sql(valid_fields)
        );

        let valid_fields: Option<&[&str]> = Some(&["id", "name"]);
        paginate_sort.sorts = vec![("id".to_owned(), Sort::Asc), ("namee".to_owned(), Sort::Desc)];
        assert_eq!(" ORDER BY id ASC".to_owned(), paginate_sort.get_sorts_sql(valid_fields));

        let valid_fields: Option<&[&str]> = Some(&["id", "name"]);
        paginate_sort.sorts = vec![("idz".to_owned(), Sort::Asc), ("namee".to_owned(), Sort::Desc)];
        assert_eq!("".to_owned(), paginate_sort.get_sorts_sql(valid_fields));
    }

    #[test]
    fn test_get_sorts_sql_with_valid_fields_and_table_prefix() {
        let valid_fields: Option<&[&str]> = Some(&["user.id", "role.name"]);
        let paginate_sort = PaginateSort {
            page: 1,
            limit: 50,
            offset: 0,
            sorts: vec![("user.id".to_owned(), Sort::Asc), ("role.name".to_owned(), Sort::Desc)],
        };
        assert_eq!(
            " ORDER BY user.id ASC, role.name DESC".to_owned(),
            paginate_sort.get_sorts_sql(valid_fields)
        );
    }
}
