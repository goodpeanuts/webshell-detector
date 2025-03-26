use diesel::table;

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
