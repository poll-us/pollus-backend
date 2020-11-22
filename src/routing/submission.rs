use super::{AuthError, AuthResult, AuthStatus};
use crate::POOL;
use gotham_restful::NoContent;
use serde::Deserialize;
use sqlx::query;

#[derive(Resource)]
#[resource(create, change)]
pub(super) struct SubmissionResource;

#[derive(Deserialize)]
struct NewSubmission {
	poll_cfg: i64,
	values: Vec<i16>
}

#[create(SubmissionResource)]
async fn create(auth: AuthStatus, body: NewSubmission) -> AuthResult<NoContent> {
	let user_id = auth.ok()?.sub;

	// only 0, 1 and 2 are allowed
	if body.values.iter().any(|v| *v > 2 || *v < 0) {
		return Err(AuthError::InvalidData);
	}

	// check that the poll config exists
	let poll_cfg = query!("SELECT id FROM poll_config WHERE id = $1;", body.poll_cfg)
		.fetch_optional(&*POOL)
		.await?;
	if poll_cfg.is_none() {
		return Err(AuthError::InvalidData);
	}

	// query stuff
	let times = query!(
		"SELECT t.time FROM poll_config_time t WHERE t.cfg = $1 ORDER BY t.time;",
		body.poll_cfg
	)
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
	query!("INSERT INTO poll_submission_time(submission, cfg, time, value) SELECT $1, $2, UNNEST($3::TIME[]), UNNEST($4::SMALLINT[]);", id, body.poll_cfg, &times, &body.values)
		.execute(&*POOL).await?;

	Ok(().into())
}

#[change(SubmissionResource)]
async fn change(auth: AuthStatus, submission: i64, values: Vec<i16>) -> AuthResult<NoContent> {
	let user_id = auth.ok()?.sub;

	// only 0, 1 and 2 are allowed
	if values.iter().any(|v| *v > 2 || *v < 0) {
		return Err(AuthError::InvalidData);
	}

	// check that the submission exists
	let sub = query!("SELECT \"user\" FROM poll_submission WHERE id = $1;", submission)
		.fetch_optional(&*POOL)
		.await?;
	if sub.is_none() {
		return Err(AuthError::NotFound);
	} else if user_id != sub.unwrap().user {
		return Err(AuthError::Forbidden);
	}

	// query stuff
	let times = query!(
		"SELECT t.time FROM poll_submission_time t WHERE t.submission = $1 ORDER BY t.time;",
		submission
	)
	.fetch_all(&*POOL)
	.await?;
	let times = times.into_iter().map(|record| record.time).collect::<Vec<_>>();

	// update stuff
	query!(
		"UPDATE poll_submission_time AS t SET value = u.value FROM (
			SELECT UNNEST($1::SMALLINT[]) AS value, UNNEST($2::TIME[]) AS time
		) AS u WHERE submission = $3 AND t.time = u.time",
		&values,
		&times,
		submission
	)
	.execute(&*POOL)
	.await?;

	Ok(().into())
}
