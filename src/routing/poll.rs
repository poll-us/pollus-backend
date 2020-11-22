use super::{AuthError, AuthResult};
use crate::POOL;
use chrono::{Duration, NaiveDate, NaiveTime};
use itertools::Itertools;
use serde::Serialize;
use sqlx::query;
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
#[resource(read)]
pub(super) struct PollResource;

#[derive(Serialize)]
struct PollEntry {
	user: i64,
	value: i16
}

#[derive(Serialize)]
struct PollConfig {
	date: NaiveDate,
	times: HashMap<NaiveTime, Vec<PollEntry>>
}

#[derive(Serialize)]
struct PollUser {
	firstname: String,
	lastname: Option<String>
}

#[derive(Serialize)]
struct Poll {
	id: String,
	cfg: HashMap<i64, PollConfig>,
	user: HashMap<i64, PollUser>
}

#[read(PollResource)]
async fn read(id: String) -> AuthResult<Poll> {
	if id.len() < 8 {
		return Err(AuthError::NotFound);
	}

	let poll_cfg_times = query!("SELECT c.id, c.date, t.time FROM poll_config c INNER JOIN poll_config_time t ON c.id = t.cfg WHERE c.poll = $1 AND c.date >= now()::DATE - 1;", &id).fetch_all(&*POOL).await?;

	// build existing config
	let mut cfg = poll_cfg_times
		.into_iter()
		.map(|record| ((record.id, record.date), record))
		.into_group_map()
		.into_iter()
		.map(|((id, date), records)| {
			(id, PollConfig {
				date,
				times: records.into_iter().map(|record| (record.time, Vec::new())).collect()
			})
		})
		.collect::<HashMap<_, _>>();

	// check that the next 7 days are present
	let dates = cfg.iter().map(|(_, cfg)| cfg.date).collect::<HashSet<_>>();
	let now = query!("SELECT now()::DATE;").fetch_one(&*POOL).await?.now.unwrap();
	for i in 0..7 {
		let day = now + Duration::days(i);
		if !dates.contains(&day) {
			let id = query!("INSERT INTO poll_config (poll, date) VALUES ($1, $2) RETURNING id;", &id, day)
				.fetch_one(&*POOL)
				.await?
				.id;
			query!("INSERT INTO poll_config_time (cfg, time) VALUES ($1, '20:00'), ($1, '21:00'), ($1, '22:00'), ($1, '23:00');", id).execute(&*POOL).await?;
			cfg.insert(id, PollConfig {
				date: day,
				times: [20, 21, 22, 23]
					.iter()
					.map(|hour| (NaiveTime::from_hms(*hour, 0, 0), Vec::new()))
					.collect()
			});
		}
	}

	// pull the data for those configs
	let cfg_ids = cfg.iter().map(|(id, _)| *id).collect::<Vec<i64>>();
	let mut user_ids: HashSet<i64> = HashSet::new();
	let data = query!(
		"SELECT s.user, s.cfg, t.value, t.time FROM poll_submission_time t INNER JOIN poll_submission s ON t.submission = s.id WHERE t.value IS NOT NULL AND s.cfg = ANY($1);",
		&cfg_ids
	).fetch_all(&*POOL).await?;
	for record in data {
		user_ids.insert(record.user);
		// the construction of cfg and above database query guarantee that nothing here is None
		cfg.get_mut(&record.cfg)
			.unwrap()
			.times
			.get_mut(&record.time)
			.unwrap()
			.push(PollEntry {
				user: record.user,
				value: record.value.unwrap() // the query guarantees this is never None
			});
	}

	// pull the required user information
	let user_ids: Vec<i64> = user_ids.into_iter().collect(); // sqlx can't deal with sets
	let user = query!(
		"SELECT u.id, u.firstname, u.lastname FROM poll_user u WHERE u.id = ANY($1);",
		&user_ids
	)
	.fetch_all(&*POOL)
	.await?;
	let user = user
		.into_iter()
		.map(|record| {
			(record.id, PollUser {
				firstname: record.firstname,
				lastname: record.lastname
			})
		})
		.collect();

	Ok(Poll { id, cfg, user })
}
