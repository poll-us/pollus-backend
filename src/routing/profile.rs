use crate::POOL;
use gotham::{
	middleware::session::SessionData,
	state::{FromState, State}
};
use serde::Serialize;
use sqlx::query;

#[derive(Resource)]
#[resource(read_all)]
pub(super) struct ProfileResource;

#[derive(Serialize)]
struct Profile {
	id: i64,
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
