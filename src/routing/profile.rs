use crate::POOL;
use gotham::{
	middleware::session::SessionData,
	state::{FromState, State}
};
use gotham_restful::NoContent;
use serde::{Deserialize, Serialize};
use sqlx::query;

#[derive(Resource)]
#[resource(read_all, change_all)]
pub(super) struct ProfileResource;

#[derive(Serialize)]
struct Profile {
	id: i64,
	firstname: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	lastname: Option<String>
}

#[derive(Deserialize)]
struct UpdateProfile {
	firstname: String,
	lastname: Option<String>
}

#[read_all(ProfileResource)]
async fn read_all(state: &mut State) -> Result<Profile, sqlx::Error> {
	let user_id: &i64 = SessionData::<i64>::borrow_from(state);
	let user = query!(
		"SELECT u.id, u.firstname, u.lastname FROM poll_user u WHERE u.id = $1;",
		user_id
	)
	.fetch_one(&*POOL)
	.await?;
	Ok(Profile {
		id: user.id,
		firstname: user.firstname,
		lastname: user.lastname
	})
}

#[change_all(ProfileResource)]
async fn change_all(state: &mut State, body: UpdateProfile) -> Result<NoContent, sqlx::Error> {
	let user_id: &i64 = SessionData::<i64>::borrow_from(state);
	query!(
		"UPDATE poll_user SET firstname = $1, lastname = $2 WHERE id = $3",
		body.firstname,
		body.lastname,
		user_id
	)
	.execute(&*POOL)
	.await?;

	Ok(Default::default())
}
