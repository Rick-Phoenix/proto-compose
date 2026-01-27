use super::*;

proto_package!(DB_TEST, name = "db_test", no_cel_test);
define_proto_file!(DB_TEST_FILE, name = "db_test.proto", package = DB_TEST);

mod schema {
  diesel::table! {
    users {
      id -> Integer,
      name -> Text,
      created_at -> Timestamp
    }
  }
}

use diesel::prelude::*;
pub use schema::users;

#[proto_message]
#[proto(skip_checks(all))]
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
  #[diesel(skip_insertion)]
  pub id: i32,
  #[proto(validate = |v| v.min_len(3))]
  pub name: String,
  #[diesel(skip_insertion)]
  #[diesel(select_expression = users::columns::created_at.nullable())]
  #[proto(timestamp)]
  pub created_at: Option<Timestamp>,
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct UserId {
  pub id: i32,
}

#[proto_service]
enum UserService {
  GetUser { request: UserId, response: User },
  InsertUser { request: User, response: Empty },
}
