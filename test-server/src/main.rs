use diesel::prelude::*;
use proto_types::Timestamp;
use test_schemas::server_models::{User, users::dsl::*};

pub mod myappv1 {
  tonic::include_proto!("db_test");
}

fn establish_connection() -> SqliteConnection {
  SqliteConnection::establish(":memory:").expect("Error connecting to the in memory db")
}

fn seed_db(conn: &mut SqliteConnection) {
  let table_query = r"
    CREATE TABLE users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
  ";

  diesel::sql_query(table_query)
    .execute(conn)
    .expect("Failed to create the table");
}

fn main() {
  let conn = &mut establish_connection();

  seed_db(conn);

  let new_user = User {
    id: 0,
    name: "Gandalf".to_string(),
    created_at: Some(Timestamp::default()),
  };

  diesel::insert_into(users)
    .values(&new_user)
    .execute(conn)
    .expect("Failed to insert user");

  let queried_user = users
    .filter(id.eq(1))
    .select(User::as_select())
    .get_result(conn)
    .expect("Failed to load user");

  println!("{queried_user:#?}");
}
