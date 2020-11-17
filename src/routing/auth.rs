use crate::POOL;
use gotham::{
	handler::HandlerError,
	helpers::http::response::create_response,
	hyper::{Body, Response, StatusCode},
	middleware::session::SessionData,
	state::{FromState, State}
};
use gotham_derive::{StateData, StaticResponseExtender};
use mime::TEXT_PLAIN;
use serde::Deserialize;
use sqlx::query;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub(super) struct AuthPath {
	token: String
}

pub(super) async fn handle_auth(state: &mut State) -> Result<Response<Body>, HandlerError> {
	let path: &AuthPath = state.borrow();

	let record = query!("SELECT u.id FROM poll_user u WHERE u.user_token = $1;", path.token)
		.fetch_one(&*POOL)
		.await?;
	let user_id: &mut i64 = SessionData::<i64>::borrow_mut_from(state);
	*user_id = record.id;

	Ok(create_response(state, StatusCode::OK, TEXT_PLAIN, "Success"))
}
