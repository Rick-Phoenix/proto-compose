#[cfg(test)]
mod test {
  use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
  use diesel::prelude::*;
  use prelude::ValidatedMessage;
  use prelude::ValidationErrors;
  use proto_types::Empty;
  use proto_types::Status as GrpcStatus;
  use test_schemas::server_models::{User, UserId, users::dsl::*};
  use tonic::{Code, Request as TonicRequest, Response as TonicResponse, Status};
  use tonic_prost::prost::Message;

  mod proto {
    tonic::include_proto!("db_test");
  }

  use proto::user_service_server::UserService as UserServiceTrait;
  use tonic_prost::prost::bytes::Bytes;

  use proto::user_service_client::UserServiceClient;
  use proto::user_service_server::UserServiceServer;

  struct UserService {
    pool: Pool,
  }

  fn handle_violations(errors: ValidationErrors) -> Status {
    let status_inner: GrpcStatus = errors.into();

    Status::with_details(
      Code::InvalidArgument,
      "Validation Error",
      Bytes::from(status_inner.encode_to_vec()),
    )
  }

  #[tonic::async_trait]
  impl UserServiceTrait for UserService {
    async fn get_user(
      &self,
      request: tonic::Request<UserId>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
      let UserId { id: user_id } = request.into_inner();

      let conn = self.pool.get().await.unwrap();

      let user = conn
        .interact(move |conn| {
          users
            .filter(id.eq(user_id))
            .select(User::as_select())
            .get_result(conn)
        })
        .await
        .map_err(|_| Status::internal("Interaction failed"))?
        .map_err(|e| Status::not_found(e.to_string()))?;

      Ok(TonicResponse::new(user))
    }

    async fn insert_user(
      &self,
      request: tonic::Request<User>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
      let msg = request
        .into_inner()
        .validated()
        .map_err(|e| handle_violations(e))?;

      let conn = self
        .pool
        .get()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

      let _ = conn
        .interact(move |conn| {
          diesel::insert_into(users)
            .values(&msg)
            .execute(conn)
        })
        .await
        .map_err(|_| Status::internal("Interaction failed"))?;

      Ok(TonicResponse::new(Empty))
    }
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

  #[tokio::test]
  async fn test() {
    let db_url = "file:test_db_1?mode=memory&cache=shared";
    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
      .max_size(1)
      .build()
      .unwrap();

    let conn = pool.get().await.unwrap();
    conn
      .interact(|conn| {
        seed_db(conn);
      })
      .await
      .unwrap();

    drop(conn);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
      .await
      .unwrap();
    let addr = listener.local_addr().unwrap();

    let service = UserService { pool: pool.clone() };

    tokio::spawn(async move {
      tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(service))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await
        .unwrap();
    });

    let mut client = UserServiceClient::connect(format!("http://{addr}"))
      .await
      .unwrap();

    let insert_req = TonicRequest::new(User {
      id: 0,
      name: "Gandalf".to_string(),
      created_at: None,
    });
    let _ = client.insert_user(insert_req).await.unwrap();

    let req = tonic::Request::new(UserId { id: 1 });
    let user = client.get_user(req).await.unwrap().into_inner();

    assert_eq!(user.name, "Gandalf");
  }
}
