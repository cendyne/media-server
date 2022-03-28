use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

no_arg_sql_function!(last_insert_rowid, sql_types::Integer);

pub fn connect_pool() -> Pool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be set in the environment to a value like database.sqlite");
    Pool::builder()
        .build(ConnectionManager::new(database_url))
        .unwrap()
}
