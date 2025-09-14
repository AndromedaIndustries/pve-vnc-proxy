use diesel::{dsl::insert_into, prelude::*};
use std::env;
use tracing::{debug, info, warn};

use crate::library::sql_lite::{
    models::{NewSession, Session},
    schema::session::dsl::*,
};

pub fn new() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_session(conn: &mut SqliteConnection, session_data: NewSession) -> Session {
    insert_into(session)
        .values(session_data)
        .returning(Session::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn get_session(conn: &mut SqliteConnection, session_id: String) -> Session {
    debug!("Getting Session {}", session_id.clone());
    let session_results = session
        .find(
            session_id
                .parse::<i32>()
                .expect("Failed to parse string to i32"),
        )
        .select(Session::as_select())
        .first(conn);

    let is_err = &session_results.is_err();

    if *is_err {
        let session_err_option = session_results.as_ref().err();
        let session_err = session_err_option.unwrap();
        warn!("Error  {:?}", session_err.to_string());
    }
    session_results.unwrap()
}

pub fn wipe(conn: &mut SqliteConnection) -> bool {
    let status = diesel::delete(session).execute(conn).is_ok();

    if status {
        return true;
    }

    false
}

pub fn delete_session(conn: &mut SqliteConnection, session_id: String) {
    let err = diesel::delete(
        session.filter(
            id.eq(session_id
                .parse::<i32>()
                .expect("Failed to parse string to i32")),
        ),
    )
    .execute(conn);

    if err.is_err() {
        warn!("Failed to delete session id {}", session_id)
    } else {
        info!("Deleted session id {}", session_id)
    }
}
