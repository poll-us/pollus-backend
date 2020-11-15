#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate gotham_restful;

use gotham::anyhow::Error;
use log::LevelFilter;
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool};
use std::{env, thread};

mod bot;
mod routing;

mod embedded {
	refinery::embed_migrations!("./migrations");
}

lazy_static! {
	static ref POOL: PgPool = PgPool::connect_lazy_with(
		env::var("DATABASE_URL")
			.map_err(Error::from)
			.and_then(|url| url.parse().map_err(Error::from))
			.map(|mut options: PgConnectOptions| {
				options.log_statements(LevelFilter::Debug);
				options
			})
			.expect("DATABASE_URL must point to a postgres database")
	);
	static ref SECRET: String = env::var("SECRET").expect("SECRET must be set to a server secret");
}

fn main() {
	pretty_env_logger::init_timed();
	if let Err(err) = dotenv::dotenv() {
		warn!("Unable to read `.env' file: {}", err);
	}

	let mut db_conf =
		refinery::config::Config::from_env_var("DATABASE_URL").expect("DATABASE_URL must point to a postgres database");
	embedded::migrations::runner()
		.run(&mut db_conf)
		.expect("Failed to run migrations");

	thread::spawn(bot::start);

	let router = routing::router();
	let addr = "[::]:7181";
	gotham::start(addr, router);
}
