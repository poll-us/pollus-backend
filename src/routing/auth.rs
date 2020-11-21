use super::AuthData;
use crate::{POOL, SECRET};
use chrono::Utc;
use cookie::Cookie;
use gotham::{
	handler::HandlerError,
	helpers::http::response::{create_empty_response, create_response},
	hyper::{header::SET_COOKIE, Body, Response, StatusCode},
	state::State
};
use gotham_derive::{StateData, StaticResponseExtender};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use mime::TEXT_PLAIN;
use serde::Deserialize;
use sqlx::query;
use time::Duration;

/// The duration a token is valid, in seconds.
const TOKEN_EXP: i64 = 60 * 60 * 24 * 30; // 30 days

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub(super) struct AuthPath {
	token: String
}

pub(super) async fn handle_auth(state: &mut State) -> Result<Response<Body>, HandlerError> {
	let path: &AuthPath = state.borrow();

	let record = query!("SELECT u.id FROM poll_user u WHERE u.user_token = $1;", path.token)
		.fetch_optional(&*POOL)
		.await?;

	let record = match record {
		Some(r) => r,
		None => return Ok(create_empty_response(state, StatusCode::FORBIDDEN))
	};

	let iat = Utc::now().timestamp();
	let data = AuthData {
		sub: record.id,
		iat,
		nbf: iat,
		exp: iat + TOKEN_EXP
	};
	let header = Header::new(Algorithm::HS256);
	let key = EncodingKey::from_secret(SECRET.as_bytes());
	let keks_wert = jsonwebtoken::encode(&header, &data, &key)?;
	let keks = Cookie::build("pollus_session", keks_wert)
		.max_age(Duration::new(TOKEN_EXP, 0))
		.path("/")
		.http_only(true)
		.secure(true)
		.finish();

	let mut res = create_response(state, StatusCode::OK, TEXT_PLAIN, "Success");
	let headers = res.headers_mut();
	headers.insert(SET_COOKIE, keks.to_string().parse()?);
	Ok(res)
}
