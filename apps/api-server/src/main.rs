extern crate api_server; // 引入 lib.rs 里面的内容
use api_server::{
    task_queue::{init_task_pool, TaskPayload},
    CtxWithLibrary,
};
use axum::routing::get;
use content_library::{load_library, upgrade_library_schemas, Library};
use dotenvy::dotenv;
use rspc::integrations::httpz::Request;
use std::{
    boxed::Box,
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Store {
    path: PathBuf,
    values: std::collections::HashMap<String, String>,
}

impl Store {
    fn new(path: PathBuf) -> Self {
        let values = std::collections::HashMap::new();
        Self { values, path }
    }
    fn load(&mut self) -> Result<(), std::io::Error> {
        let file = std::fs::File::open(&self.path)?;
        let reader = std::io::BufReader::new(file);
        let values: std::collections::HashMap<String, String> = serde_json::from_reader(reader)?;
        self.values = values;
        Ok(())
    }
    fn save(&self) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(&self.path)?;
        serde_json::to_writer(file, &self.values)?;
        Ok(())
    }
    fn insert(&mut self, key: &str, value: &str) -> Result<(), ()> {
        self.values.insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
}

#[derive(Clone)]
struct Ctx {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<Store>>,
    current_library: Arc<Mutex<Option<Library>>>,
    tx: Arc<broadcast::Sender<TaskPayload>>,
}

impl CtxWithLibrary for Ctx {
    fn get_local_data_root(&self) -> PathBuf {
        self.local_data_root.clone()
    }
    fn get_resources_dir(&self) -> PathBuf {
        self.resources_dir.clone()
    }
    fn library(&self) -> Result<Library, rspc::Error> {
        match self.current_library.lock().unwrap().as_ref() {
            Some(library) => Ok(library.clone()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No current library is set"),
            )),
        }
    }
    fn switch_current_library<'async_trait>(
        &'async_trait self,
        library_id: &'async_trait str,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        let mut store = self.store.lock().unwrap();
        let _ = store.insert("current-library-id", library_id);
        let _ = store.save();
        // try to load library, but this is not necessary
        let _ = store.load();
        if let Some(library_id) = store.get("current-library-id") {
            let library_id = library_id.clone();
            return Box::pin(async move {
                let library = load_library(&self.local_data_root, &library_id).await;
                self.current_library.lock().unwrap().replace(library);
                info!("Current library switched to {}", library_id);
            });
        } else {
            // 这里实际上不可能被执行，除非 settings.json 数据有问题
            return Box::pin(async move {
                self.current_library.lock().unwrap().take();
                info!("Current library is unset");
            });
        }
    }
    fn get_task_tx(&self) -> Arc<broadcast::Sender<TaskPayload>> {
        Arc::clone(&self.tx)
    }
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };
    init_tracing(); // should be after dotenv() so RUST_LOG in .env file will be loaded
                    // debug!("test debug output");
    let local_data_root = match env::var("LOCAL_DATA_DIR") {
        Ok(path) => Path::new(&path).to_path_buf(),
        Err(_e) => {
            // #[cfg(not(debug_assertions))]
            // {}
            panic!("'$LOCAL_DATA_DIR' is not set ({})", _e)
        }
    };
    std::fs::create_dir_all(&local_data_root).unwrap();

    let resources_dir = match env::var("LOCAL_RESOURCES_DIR") {
        Ok(path) => Path::new(&path).to_path_buf(),
        Err(_e) => {
            panic!("'$LOCAL_RESOURCES_DIR' is not set ({})", _e)
        }
    };
    // let resources_dir = local_data_root.join("resources").to_str().unwrap().to_owned();
    // let resources_dir = Path::new(&resources_dir).to_path_buf();

    upgrade_library_schemas(&local_data_root).await;

    let tx = init_task_pool();
    let router = api_server::router::get_router::<Ctx>();

    let store = Arc::new(Mutex::new(Store::new(
        local_data_root.join("settings.json"),
    )));
    let current_library = Arc::new(Mutex::<Option<Library>>::new(None));
    {
        let mut store_mut = store.lock().unwrap();
        let _ = store_mut.load();
        if let Some(library_id) = store_mut.get("current-library-id") {
            let library = load_library(&local_data_root, &library_id).await;
            current_library.lock().unwrap().replace(library);
        }
    }

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);
    let app: axum::Router = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            router
                .clone()
                .endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
                    Ctx {
                        local_data_root,
                        resources_dir,
                        store,
                        current_library,
                        tx,
                    }
                })
                .axum(),
        )
        // .nest_service("/artifacts", ServeDir::new(local_data_dir.clone()))
        .nest_service("/file/localhost", ServeDir::new("/"))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            // load filters from the `RUST_LOG` environment variable.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();
}
