use tracing_log::LogTracer;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_subscriber::fmt::MakeWriter;

pub fn get_tracing_subscriber(name: String, log_level: String, sink: impl MakeWriter + Send + Sync + 'static) -> impl Subscriber + Send + Sync {
	let env_filter = EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| EnvFilter::new(log_level));

	let formatting_layer = BunyanFormattingLayer::new(
		name,
		sink
	);

	Registry::default()
		.with(env_filter)
		.with(JsonStorageLayer)
		.with(formatting_layer)
}

pub fn init_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
	LogTracer::init().expect("Failed to set log tracer");
	set_global_default(subscriber).expect("Failed to set subscriber");
}

