use std::{
    io::prelude::*,
    path::PathBuf,
    sync::Mutex,
};
use dotenvy::dotenv;
// use tracing::subscriber;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

// mod open_telemetry;
// use open_telemetry::init_otel_layer;


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

    // let telemetry_layer = init_otel_layer();

    tracing_subscriber::registry()
        .with(env_layer)
        .with(stdout_layer)
        // .with(telemetry_layer)
        .init();
}

pub fn init_tracing_to_file(log_dir: PathBuf) {
    let log_file_full_path = log_dir.join(format!("app.{}.log", chrono::Utc::now().format("%Y-%m-%d")));

    std::panic::set_hook({
        let log_file_full_path = log_file_full_path.clone();
        Box::new(move |panic_info| {
            let mut file = match std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&log_file_full_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create log file: {}", e);
                    return;
                }
            };
            let _ = writeln!(file, "Panic: {}", panic_info);
            // if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            //     let _ = writeln!(file, "Panic: {}", s);
            // } else {
            //     let _ = writeln!(file, "Panic: Unknown");
            // }
            // if let Some(location) = panic_info.location() {
            //     let _ = writeln!(file, "Occurred at: {}:{}", location.file(), location.line());
            // }
        })
    });

    let env_layer = init_env_layer();

    // let telemetry_layer = init_otel_layer();

    /*
    * see logs with cmd:
    * log stream --debug --predicate 'subsystem=="ai.gendam.desktop" and category=="default"'
    */
    // let os_logger = tracing_oslog::OsLogger::new("ai.gendam.desktop", "default");

    let file_log_layer = {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log dir: {}", e);
            init_tracing_to_stdout();  // fallback to stdout tracing
            return;
        }
        let file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file_full_path)
        {
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
        // .with(telemetry_layer)
        .init();
}
