use crate::POOL;
use gotham::{
	middleware::session::SessionData,
	state::{FromState, State}
};
use gotham_restful::NoContent;
use serde::Deserialize;
use sqlx::query;

#[derive(Resource)]
#[resource(create)]
pub(super) struct SubmissionResource;

#[derive(Deserialize)]
struct NewSubmission {
	poll_cfg: i64,
	values: Vec<i16>
}

#[create(SubmissionResource)]
async fn create(state: &mut State, body: NewSubmission) -> Result<NoContent, sqlx::Error> {
	let user_id: &i64 = SessionData::<i64>::borrow_from(state);

	// query stuff
	let times = query!("SELECT t.time FROM poll_config_time t WHERE t.cfg = $1;", body.poll_cfg)
		.fetch_all(&*POOL)
		.await?;
	let times = times.into_iter().map(|record| record.time).collect::<Vec<_>>();

	// insert stuff
	let id = query!(
		"INSERT INTO poll_submission(cfg, \"user\") VALUES ($1, $2) RETURNING id;",
		body.poll_cfg,
		user_id
	)
	.fetch_one(&*POOL)
	.await?
	.id;
	// TODO there is a bug with query! that makes this not compile
	query("INSERT INTO poll_submission_time(submission, cfg, time, value) SELECT $1, $2, UNNEST($3::TIME[]), UNNEST($4::SMALLINT[]);")
		.bind(id).bind(body.poll_cfg).bind(&times).bind(&body.values).execute(&*POOL).await?;

	Ok(().into())
}
