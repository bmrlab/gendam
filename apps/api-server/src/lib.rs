use rspc::Rspc;

#[derive(Clone)]
pub struct Ctx {
    pub x_demo_header: Option<String>,
}
pub const R: Rspc<Ctx> = Rspc::new();

pub mod routes;
pub mod router;
