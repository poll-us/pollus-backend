use crate::SECRET;
use gotham::{
	hyper::header::CONTENT_TYPE,
	middleware::{cookie::CookieParser, logger::RequestLogger},
	pipeline::{new_pipeline, single::single_pipeline},
	router::{builder::*, Router}
};
use gotham_restful::{AuthMiddleware, AuthSource, AuthValidation, CorsConfig, DrawResources, Origin, StaticAuthHandler};
use log::Level;
use serde::{Deserialize, Serialize};

mod auth;
mod poll;
mod profile;
mod submission;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AuthData {
	sub: i64,
	iat: i64,
	nbf: i64,
	exp: i64
}

type AuthStatus = gotham_restful::AuthStatus<AuthData>;

#[derive(Debug, ResourceError)]
enum AuthError {
	#[status(FORBIDDEN)]
	#[display("Forbidden")]
	Forbidden,
	#[status(BAD_REQUEST)]
	#[display("Invalid Data")]
	InvalidData,
	#[status(NOT_FOUND)]
	#[display("Not Found")]
	NotFound,
	#[status(INTERNAL_SERVER_ERROR)]
	#[display("{0}")]
	DatabaseError(sqlx::Error)
}

impl From<gotham_restful::AuthError> for AuthError {
	fn from(_: gotham_restful::AuthError) -> Self {
		Self::Forbidden
	}
}

impl From<sqlx::Error> for AuthError {
	fn from(err: sqlx::Error) -> Self {
		Self::DatabaseError(err)
	}
}

type AuthResult<T> = Result<T, AuthError>;

pub(crate) fn router() -> Router {
	let logger = RequestLogger::new(Level::Info);

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: vec![CONTENT_TYPE],
		credentials: true,
		..Default::default()
	};

	let kekse = CookieParser;

	let auth: AuthMiddleware<AuthData, _> = AuthMiddleware::new(
		AuthSource::Cookie("pollus_session".to_owned()),
		AuthValidation::default(),
		StaticAuthHandler::from_array(SECRET.as_bytes())
	);

	let (chain, pipelines) = single_pipeline(new_pipeline().add(logger).add(cors).add(kekse).add(auth).build());
	build_router(chain, pipelines, |route| {
		route
			.get("/auth/:token")
			.with_path_extractor::<auth::AuthPath>()
			.to_async_borrowing(auth::handle_auth);

		route.resource::<poll::PollResource>("/poll");
		route.resource::<profile::ProfileResource>("/profile");
		route.resource::<submission::SubmissionResource>("/submission");
	})
}
