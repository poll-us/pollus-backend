use crate::POOL;
use chrono::{Duration, NaiveDate, NaiveTime};
use gotham_restful::Success;
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
	name: String,
	submissions: Vec<PollSubmission>
}

#[derive(Serialize)]
struct PollSubmission {
	id: i64,
	values: Vec<i8>
}

#[derive(Serialize)]
struct Poll {
	id: String,
	cfg: Vec<PollConfig>
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

	Ok(Poll { id, cfg })
}
