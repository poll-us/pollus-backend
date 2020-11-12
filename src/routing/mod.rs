use gotham::{
	handler::HandlerError,
	helpers::http::response::create_empty_response,
	hyper::{
		header::{CONTENT_TYPE, LOCATION},
		Body, Response, StatusCode
	},
	middleware::{
		logger::RequestLogger,
		session::{NewSessionMiddleware, SessionData}
	},
	pipeline::{new_pipeline, single::single_pipeline},
	router::{builder::*, Router},
	state::{FromState, State}
};
use gotham_restful::{CorsConfig, DrawResources, Origin};
use log::Level;

mod poll;

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
	})
}
