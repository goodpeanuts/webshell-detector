use diesel::prelude::*;

table! {
    token (token_column) {
        #[sql_name = "token"]
        token_column -> Text,
        len -> Integer,
        level -> Integer,
    }
}

table! {
    preg (preg_column) {
        #[sql_name = "Preg"]
        preg_column -> Text,
        level -> Integer,
    }
}

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

pub fn establish_connection() -> SqliteConnection {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    assert!(
        std::path::Path::new(&database_url).exists(),
        "Database file not found"
    );
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
