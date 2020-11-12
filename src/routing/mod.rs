use gotham::{
	hyper::header::CONTENT_TYPE,
	middleware::logger::RequestLogger,
	pipeline::{new_pipeline, single::single_pipeline},
	router::{builder::*, Router}
};
use gotham_restful::{CorsConfig, DrawResources, Origin};
use log::Level;

mod poll;
mod submission;

pub(crate) fn router() -> Router {
	let logger_middleware = RequestLogger::new(Level::Info);

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: vec![CONTENT_TYPE],
		..Default::default()
	};

	let (chain, pipelines) = single_pipeline(new_pipeline().add(logger_middleware).add(cors).build());
	build_router(chain, pipelines, |route| {
		route.resource::<poll::PollResource>("/poll");
		route.resource::<submission::SubmissionResource>("/submission");
	})
}
