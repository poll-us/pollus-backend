use gotham::{
	hyper::header::CONTENT_TYPE,
	middleware::{logger::RequestLogger, session::NewSessionMiddleware},
	pipeline::{new_pipeline, single::single_pipeline},
	router::{builder::*, Router}
};
use gotham_restful::{CorsConfig, DrawResources, Origin};
use log::Level;

mod auth;
mod poll;
mod profile;
mod submission;

pub(crate) fn router() -> Router {
	let logger = RequestLogger::new(Level::Info);

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: vec![CONTENT_TYPE],
		..Default::default()
	};

	let sessions = NewSessionMiddleware::default().with_session_type::<i64>();

	let (chain, pipelines) = single_pipeline(new_pipeline().add(logger).add(cors).add(sessions).build());
	build_router(chain, pipelines, |route| {
		route
			.get("/auth/:token")
			.with_path_extractor::<auth::AuthPath>()
			.to_async_borrowing(auth::handle_auth);

		route.resource::<poll::PollResource>("/poll");
		route.resource::<profile::ProfileResource>("/profile");
		route.resource::<submission::SubmissionResource>("/submission");
	})
}
