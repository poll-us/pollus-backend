use crate::POOL;
use chrono::{Duration, NaiveDate, NaiveTime};
use itertools::Itertools;
use serde::Serialize;
use sqlx::query;
use std::collections::HashSet;

#[derive(Resource)]
#[resource(read)]
pub(super) struct PollResource;

#[derive(Serialize)]
struct PollConfig {
	id: i64,
	date: NaiveDate,
	times: Vec<NaiveTime>
}

#[derive(Serialize)]
struct PollData {
	id: i64,
	firstname: String,
	lastname: Option<String>,
	submissions: Vec<PollSubmission>
}

#[derive(Serialize)]
struct PollSubmission {
	id: i64,
	time: NaiveTime,
	value: i16
}

#[derive(Serialize)]
struct Poll {
	id: String,
	cfg: Vec<PollConfig>,
	data: Vec<PollData>
}

#[read(PollResource)]
async fn read(id: String) -> Result<Poll, sqlx::Error> {
	let poll_cfg_times = query!("SELECT c.id, c.date, t.time FROM poll_config c INNER JOIN poll_config_time t ON c.id = t.cfg WHERE c.poll = $1 AND c.date >= now()::DATE - 1;", &id).fetch_all(&*POOL).await?;

	// build existing config
	let mut cfg = poll_cfg_times
		.into_iter()
		.map(|record| ((record.id, record.date), record))
		.into_group_map()
		.into_iter()
		.map(|((id, date), records)| PollConfig {
			id,
			date,
			times: records.into_iter().map(|record| record.time).collect()
		})
		.collect::<Vec<_>>();

	// check that the next 7 days are present
	let dates = cfg.iter().map(|cfg| cfg.date).collect::<HashSet<_>>();
	let now = query!("SELECT now()::DATE;").fetch_one(&*POOL).await?.now.unwrap();
	for i in 0..7 {
		let day = now + Duration::days(i);
		if !dates.contains(&day) {
			let id = query!("INSERT INTO poll_config (poll, date) VALUES ($1, $2) RETURNING id;", &id, day)
				.fetch_one(&*POOL)
				.await?
				.id;
			query!("INSERT INTO poll_config_time (cfg, time) VALUES ($1, '20:00'), ($1, '21:00'), ($1, '22:00'), ($1, '23:00');", id).execute(&*POOL).await?;
			cfg.push(PollConfig {
				id,
				date: day,
				times: [20, 21, 22, 23].iter().map(|hour| NaiveTime::from_hms(*hour, 0, 0)).collect()
			});
		}
	}

	// pull the data for those configs
	let cfg_ids = cfg.iter().map(|cfg| cfg.id).collect::<Vec<i64>>();
	let data = query!(
		"SELECT u.id, u.firstname, u.lastname, t.submission, t.value, t.time FROM poll_submission_time t INNER JOIN poll_submission s ON t.submission = s.id INNER JOIN poll_user u ON s.user = u.id WHERE t.value IS NOT NULL AND s.cfg = ANY($1);",
		&cfg_ids
	).fetch_all(&*POOL).await?;
	let data = data
		.into_iter()
		.map(|record| {
			(
				(record.id, record.firstname, record.lastname),
				(record.submission, record.value, record.time)
			)
		})
		.into_group_map()
		.into_iter()
		.map(|((id, firstname, lastname), submissions)| PollData {
			id,
			firstname,
			lastname,
			submissions: submissions
				.into_iter()
				.map(|(id, value, time)| PollSubmission {
					id,
					// the database query guarantees that this is never null
					value: value.unwrap(),
					time
				})
				.collect()
		})
		.collect();

	Ok(Poll { id, cfg, data })
}
