// use std::sync::Arc;

// use prisma_client_rust::QueryError;
// use prisma_lib::PrismaClient;

// use crate::routes::p2p::generated::structs::VideoInfo;

// // 用于在分享完文件后更新相应的数据库
// // 可能未来要根据 对方的版本和自己的版本调用不同的函数，保证数据库写入成功
// pub async fn update_db(
//     prisma_client: Arc<PrismaClient>,
//     video_info: VideoInfo,
// ) -> Result<(), QueryError> {
//     let hash = video_info.hash.expect("hash is required");
//     let file_name = video_info.file_name.expect("file_name is required");
//     todo!("update db")
// }
