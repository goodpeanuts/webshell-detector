use diesel::prelude::*;

#[allow(dead_code)]
#[derive(QueryableByName, Debug)]
pub(crate) struct TableName {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(QueryableByName, Debug)]
pub(crate) struct Token {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) token: String,

    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(crate) len: i32,

    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(crate) level: i32,
}

#[derive(QueryableByName, Debug)]
pub(crate) struct Preg {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) preg: String,

    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(crate) level: i32,
}
