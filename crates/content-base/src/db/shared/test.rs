use rand::Rng;

use crate::db::DB;

pub async fn setup() -> DB {
    dotenvy::dotenv().ok();
    DB::new().await
}

pub fn gen_vector(size: usize) -> Vec<f32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(0.0..1.0))
        .collect()
}
