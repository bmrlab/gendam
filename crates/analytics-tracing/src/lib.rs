use std::{
    path::PathBuf,
    sync::Mutex,
};
use dotenvy::dotenv;
// use tracing::subscriber;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

mod open_telemetry;
use open_telemetry::init_otel_layer;


fn init_env_layer() -> tracing_subscriber::EnvFilter {
    match dotenv() {
        Ok(path) => eprintln!(".env read successfully from {}", path.display()),
        Err(e) => eprintln!("Could not load .env file: {e}"),
    };

    let env_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "debug".into());
        // .unwrap_or_else(|_| "api_server=debug,muse_desktop=debug".into());

    return env_layer;
}

pub fn init_tracing_to_stdout() {
    let env_layer = init_env_layer();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true);

    let telemetry_layer = init_otel_layer();

    tracing_subscriber::registry()
        .with(env_layer)
        .with(stdout_layer)
        .with(telemetry_layer)
        .init();
}

pub fn init_tracing_to_file(log_dir: PathBuf) {
    let env_layer = init_env_layer();

    let telemetry_layer = init_otel_layer();

    /*
    * see logs with cmd:
    * log stream --debug --predicate 'subsystem=="cc.musedam.local" and category=="default"'
    */
    // let os_logger = tracing_oslog::OsLogger::new("cc.musedam.local", "default");

    let file_log_layer = {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log dir: {}", e);
            init_tracing_to_stdout();  // fallback to stdout tracing
            return;
        }
        let file = match std::fs::File::create(log_dir.join("app.log")) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create log file: {}", e);
                init_tracing_to_stdout();  // fallback to stdout tracing
                return;
            }
        };
        tracing_subscriber::fmt::layer()
            .with_writer(Mutex::new(file))
            .with_ansi(false)
    };

    tracing_subscriber::registry()
        .with(env_layer)
        .with(file_log_layer)
        // .with(os_logger)
        .with(telemetry_layer)
        .init();
}
