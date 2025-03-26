use diesel::prelude::*;
use diesel::Queryable;

#[allow(dead_code)]
#[derive(QueryableByName, Debug)]
pub(crate) struct TableName {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(Queryable, Debug)]
pub(crate) struct Token {
    pub(crate) token: String,
    pub(crate) len: i32,
    pub(crate) level: i32,
}

#[derive(Queryable, Debug)]
pub(crate) struct Preg {
    pub(crate) preg: String,
    pub(crate) level: i32,
}
