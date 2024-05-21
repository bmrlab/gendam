
pub fn sql_error(e: prisma_client_rust::QueryError) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("sql query failed: {}", e),
    )
}
