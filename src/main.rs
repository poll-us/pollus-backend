#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate gotham_restful;

use sqlx::PgPool;
use std::{env, thread};

mod bot;
mod routing;

mod embedded {
	refinery::embed_migrations!("./migrations");
}

lazy_static! {
	static ref POOL: PgPool =
		PgPool::connect_lazy(&env::var("DATABASE_URL").expect("DATABASE_URL must point to a postgres database"))
			.expect("Failed to connect to database");
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
