use diesel::{prelude::QueryableByName, sqlite::SqliteConnection, Connection};
use std::env;
use dotenvy::dotenv;

fn establish_connection() -> SqliteConnection {
    dotenv().ok(); // 读取 .env 文件
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(QueryableByName)]
struct TableName {
    #[sql_type = "diesel::sql_types::Text"]
    name: String,
}

fn list_tables(conn: &mut SqliteConnection) {
    use diesel::prelude::*;
    let query = "SELECT name FROM sqlite_master WHERE type='table'";
    let results: Vec<TableName> = diesel::sql_query(query)
        .load(conn)
        .expect("Failed to fetch table names");

    println!("Tables in the database:");
    for table in results {
        println!("- {}", table.name);
    }
}

fn main() {
    let mut conn = establish_connection();
    println!("Connected to SQLite database at db/data.db!");
    list_tables(&mut conn);
}