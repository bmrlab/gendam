use content_library::Library;
use futures::AsyncWriteExt;
use p2p::{str_to_peer_id, Node};
use p2p_block::{StreamData, Transfer, TransferRequest};
use prisma_lib::{asset_object, file_path, PrismaClient};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};
use tokio::io::BufWriter;
use uuid::Uuid;
pub mod file;
pub mod folder;
use crate::{routes::create::create_asset_object, utils::path::get_suffix_path, ShareInfo};
use autosurgeon::{Hydrate, Reconcile};

// 存放文件属性，比如tag，info信息
#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq)]
pub struct File {
    pub name: String,
    pub hash: String,
}

impl File {
    pub async fn save_to_db(
        &self,
        library: Arc<Library>,
        doc_id: String,
    ) -> Result<(), anyhow::Error> {
        let new_name = self.name.clone();
        // 更新filepath数据库
        let _ = library
            .prisma_client()
            .file_path()
            .update(
                file_path::UniqueWhereParam::DocIdEquals(doc_id.clone()),
                vec![file_path::name::set(new_name)],
            )
            .exec()
            .await
            .expect("update file_path error");
        Ok(())
    }
}

#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub name: String,
    pub children: Vec<Item>,
}
#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub doc_id: Option<String>, // 现在新增文件都会有文档id，只有文件夹可能没有
    pub path: String,
    pub is_dir: bool,
    pub hash: Option<String>, // 处理新增文件用的
}

impl Folder {
    // 写入数据库
    pub async fn save_to_db(
        &self,
        library: Arc<Library>,
        doc_id: String,
        node: Arc<Node<ShareInfo>>,
        peer_id: String,
    ) -> Result<(), anyhow::Error> {
        // 尝试hydrate folder
        // 说明这是更新某个文件夹的文档
        // 处理共享文件夹的名字，和子文件夹的名字，新增子项，删除子项
        let file_path_data = library
            .prisma_client()
            .file_path()
            .find_unique(file_path::UniqueWhereParam::DocIdEquals(doc_id.clone()))
            .exec()
            .await
            .expect("find file_path error")
            .unwrap(); // 一定有

        // 1. 更新共享文件夹名字
        if file_path_data.name != self.name {
            let _ = library
                .prisma_client()
                .file_path()
                .update(
                    file_path::UniqueWhereParam::DocIdEquals(doc_id.clone()),
                    vec![file_path::name::set(self.name.clone())],
                )
                .exec()
                .await
                .expect("update file_path error");
        }

        // 找到folder下面所有路径
        let root_path = file_path_data.materialized_path.clone() + &self.name.clone() + "/";

        let children = Folder::from_db(
            library.prisma_client(),
            file_path_data.materialized_path.clone(),
            self.name.clone(),
        )
        .await?
        .children;

        tracing::debug!("children: {children:?}");
        /*
            比对 children 和 folder.children的差异
            1. 新增了文件夹
            2. 新增了文件
            3. 删除了文件夹
            4. 删除了文件
            5. 更新了文件夹名字
            6. 更新了文件的名字
            7. 移动了文件夹
            8. 移动了文件
        */
        // 更新也是删除了，再创建新的
        // 处理删除
        for sub_file in children.clone() {
            let mut is_delete = true;
            for child in self.children.clone() {
                if sub_file.path == child.path {
                    is_delete = false;
                    break;
                }
            }

            if is_delete {
                tracing::debug!("{:?} 需要删除", sub_file.path);
                // 如果这个文件是上传完了，但任务没有完成的文件，就忽略这个文件
                // 查这个任务
                if !sub_file.is_dir {
                    let hash = sub_file.hash.unwrap();
                    let asset_object_data = library
                        .prisma_client()
                        .asset_object()
                        .find_unique(asset_object::UniqueWhereParam::HashEquals(hash))
                        .with(asset_object::tasks::fetch(vec![]))
                        .exec()
                        .await?;
                    if let Some(asset_object_data) = asset_object_data {
                        if let Some(tasks) = asset_object_data.tasks {
                            let mut is_done = true;
                            for task in tasks {
                                let exit_code = task.exit_code;
                                match exit_code {
                                    Some(code) => {
                                        if code < 0 {
                                            is_done = false
                                        }
                                    }
                                    None => is_done = false,
                                }
                            }
                            if !is_done {
                                // 不做修改
                                return Ok(());
                            }
                        }
                    }
                }

                // 获取文件路径和name
                let need_delete_materialized_name =
                    String::from(sub_file.path.clone().split("/").last().unwrap());

                let need_delete_materialized_path = sub_file.path.clone();
                // 前面的路径 need_delete_materialized_path 去掉最后一个名字
                let need_delete_materialized_path_list = need_delete_materialized_path
                    .split("/")
                    .collect::<Vec<&str>>();

                let need_delete_materialized_path_suffix_list = need_delete_materialized_path_list
                    .iter()
                    .take(need_delete_materialized_path_list.len() - 1)
                    .collect::<Vec<&&str>>();

                let mut need_delete_materialized_path_suffix_string = String::new();
                for suffix in need_delete_materialized_path_suffix_list {
                    need_delete_materialized_path_suffix_string.push_str("/");
                    need_delete_materialized_path_suffix_string.push_str(suffix);
                }

                let need_delete_materialized_path = format!(
                    "{}{}{}/",
                    file_path_data.materialized_path.clone(),
                    self.name.clone(),
                    need_delete_materialized_path_suffix_string.clone()
                );

                tracing::debug!(
                    "need_delete_materialized_path: {need_delete_materialized_path:?}, name: {need_delete_materialized_name:?}"
                );
                let _ = library
                    .prisma_client()
                    .file_path()
                    .delete(file_path::UniqueWhereParam::MaterializedPathNameEquals(
                        need_delete_materialized_path.clone(),
                        need_delete_materialized_name.clone(),
                    ))
                    .exec()
                    .await
                    .unwrap();
                // todo 删除文件还需要删除源文件和artifact (也不一定，因为移动的话就要重新来一遍)
            }
        }

        // 处理新增
        for child in self.children.clone() {
            let mut is_new = true;
            for sub_file in children.clone() {
                if child.path == sub_file.path {
                    is_new = false;
                    break;
                }
            }

            if is_new {
                tracing::debug!("{:?} 需要增加", child.path);
                if child.is_dir {
                    // 新增了文件夹
                    tracing::debug!("新增了文件夹: {child:#?}");

                    let child_path = child.path.clone();
                    let new_folder_path = Path::new(&child_path);
                    let new_folder_path_split = new_folder_path.components().collect::<Vec<_>>();
                    let new_folder_name = new_folder_path_split
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap()
                        .to_string();

                    tracing::debug!("new_folder_name: {new_folder_name:?}");

                    let mut new_folder_relation_path = String::new();
                    for i in 0..new_folder_path_split.len() - 1 {
                        new_folder_relation_path.push_str("/");
                        new_folder_relation_path
                            .push_str(new_folder_path_split[i].as_os_str().to_str().unwrap());
                    }
                    let new_folder_materialized_path = format!(
                        "{}{}{}/",
                        file_path_data.materialized_path.clone(),
                        self.name.clone(),
                        new_folder_relation_path
                    );
                    tracing::debug!(
                        "new_folder_materialized_path: {new_folder_materialized_path:?}"
                    );

                    // 创建新文件夹
                    let _ = library
                        .prisma_client()
                        .file_path()
                        .create(true, new_folder_materialized_path, new_folder_name, vec![])
                        .exec()
                        .await
                        .unwrap();
                } else {
                    // 新增了文件
                    tracing::debug!("新增了文件: {child:#?}");
                    // 传输文件并写入数据库
                    let new_file_doc_id = child.doc_id.clone().unwrap();
                    // 先查有没有这个docid 如果别的文档创建了，这里就可以停止了
                    if let None = library
                        .prisma_client()
                        .file_path()
                        .find_unique(file_path::UniqueWhereParam::DocIdEquals(
                            new_file_doc_id.clone(),
                        ))
                        .exec()
                        .await
                        .unwrap()
                    {
                        // 没有这条数据
                        let new_file_path = Path::new(&child.path);
                        let new_file_path_split = new_file_path.components().collect::<Vec<_>>();
                        let new_file_name = new_file_path_split
                            .last()
                            .unwrap()
                            .as_os_str()
                            .to_str()
                            .unwrap()
                            .to_string();
                        let mut new_file_materialized_path = format!("{}", root_path.clone());
                        for i in 0..new_file_path_split.len() - 1 {
                            new_file_materialized_path
                                .push_str(new_file_path_split[i].as_os_str().to_str().unwrap());
                            new_file_materialized_path.push_str("/");
                        }
                        tracing::debug!("新增的文件的 new_file_materialized_path:{new_file_materialized_path:?}, new_file_name:{new_file_name:?}");

                        // 对方文件的hash
                        let hash = child.hash.clone().unwrap();

                        match library
                            .prisma_client()
                            .asset_object()
                            .find_unique(asset_object::UniqueWhereParam::HashEquals(hash.clone()))
                            .exec()
                            .await
                            .unwrap()
                        {
                            Some(asset_object_data) => {
                                // 说明有这个文件资源
                                // 不需要索要了，todo用fs查一下
                                library
                                    .prisma_client()
                                    .file_path()
                                    .create(
                                        false,
                                        new_file_materialized_path,
                                        new_file_name,
                                        vec![
                                            file_path::doc_id::set(Some(new_file_doc_id.clone())),
                                            file_path::asset_object_id::set(Some(
                                                asset_object_data.id,
                                            )),
                                        ],
                                    )
                                    .exec()
                                    .await
                                    .unwrap();
                            }
                            None => {
                                // 没有这个资源
                                // 更新数据库 这里就不能添加 asset_object 信息
                                // 这里就要向对方索要文件数据了
                                // 这里发送一个事件，在别的事件里做这个事情
                                let id = Uuid::new_v4();
                                let event = p2p_block::message::Message::<
                                    p2p_block::RequestDocument,
                                >::RequestDocument(
                                    p2p_block::RequestDocument {
                                        id,
                                        hash: hash.clone(),
                                    },
                                );

                                // 开启stream
                                let mut stream = node
                                    .open_stream(str_to_peer_id(peer_id.clone()).unwrap())
                                    .await
                                    .expect("open stream fail");

                                // 发送事件
                                let bytes = event.to_bytes();

                                stream.write_all(&bytes).await.unwrap();

                                // 等待对方发送文件
                                tracing::debug!("等待对方发送文件");

                                // 这里等待对方发送文件信息
                                if let Ok(requests) =
                                    TransferRequest::<String>::from_stream(&mut stream).await
                                {
                                    // 同意发过来
                                    let cancelled = Arc::new(AtomicBool::new(false));

                                    let mut transfer = Transfer::new(
                                        &requests,
                                        |percent| {
                                            tracing::info!("{id} 接收文件进度{percent}%");
                                        },
                                        &cancelled,
                                    );

                                    // 临时文件夹
                                    let target_file_path = library
                                        .get_temp_dir()
                                        .unwrap()
                                        .join(Uuid::new_v4().to_string());

                                    match tokio::fs::File::create(&target_file_path).await {
                                        Ok(f) => {
                                            tracing::info!(
                                                "({id}): create file at '{target_file_path:?}'"
                                            );
                                            let f: BufWriter<tokio::fs::File> = BufWriter::new(f);
                                            if let Err(err) = transfer.receive(&mut stream, f).await
                                            {
                                                tracing::error!(
                                                    "({id}): error receiving file: '{err:?}'"
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            tracing::error!("({id}): error creating file at '{target_file_path:?}': '{err:?}'");
                                        }
                                    }

                                    // 解压，放到对应的位置上
                                    tracing::info!("({id}): 解压文件");
                                    let file_hashes =
                                        library.unpack_bundle(&target_file_path).unwrap();
                                    // 解压完毕
                                    tracing::debug!("file_hashes: {:?}", file_hashes);
                                    // 写入数据库
                                    let (_file_path_data, _asset_object_data, _asset_object_existed) =
                                        create_asset_object(
                                            &library,
                                            &new_file_materialized_path,
                                            &new_file_name,
                                            &library
                                                .file_path(&hash)
                                                .to_string_lossy()
                                                .to_string()
                                                .as_str(),
                                        )
                                        .await?;
                                    tracing::debug!("{hash:?} 写入完成",);
                                };
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // 从数据库生成
    pub async fn from_db(
        prisma_client: Arc<PrismaClient>,
        materialized_path: String,
        name: String,
    ) -> Result<Self, anyhow::Error> {
        // 使用 tree_map 排序 路径和is_dir做key
        let mut tree_map: BTreeMap<(String, bool), Item> = BTreeMap::new();
        let root_path = materialized_path.clone() + &name.clone() + "/";
        let prefix = materialized_path.clone() + &name.clone();
        tracing::debug!("root_path:{root_path:?}");
        let sub_file_res = prisma_client
            .file_path()
            .find_many(vec![file_path::materialized_path::starts_with(
                root_path.clone(),
            )])
            .with(file_path::asset_object::fetch())
            .exec()
            .await
            .unwrap();

        tracing::debug!("sub_file_res: {sub_file_res:?}");

        for sub_file in sub_file_res {
            let suffix = get_suffix_path(
                &format!(
                    "{}{}",
                    sub_file.materialized_path.clone(),
                    &sub_file.name.clone()
                ),
                &prefix.clone(),
            );

            let item = Item {
                path: suffix,
                is_dir: sub_file.is_dir,
                doc_id: sub_file.doc_id,
                hash: match sub_file.is_dir {
                    true => None,
                    false => Some(sub_file.asset_object.unwrap().unwrap().hash),
                },
            };

            tree_map.insert((item.path.clone(), item.is_dir), item);
        }

        tracing::debug!("tree_map: {tree_map:?}");

        Ok(Folder {
            name,
            children: tree_map.values().cloned().collect(),
        })
    }

    pub fn push(&mut self, item: Item) -> () {
        // 使用 tree_map 排序
        let mut tree_map: BTreeMap<(String, bool), Item> = BTreeMap::new();
        for child in self.children.clone() {
            tree_map.insert((child.path.clone(), item.is_dir), child);
        }
        tree_map.insert((item.path.clone(), item.is_dir), item);
        self.children = tree_map.values().cloned().collect();
    }

    pub fn remove(&mut self, relation_path: String, is_dir: bool) -> () {
        // 分2种情况，文件夹和文件
        if is_dir {
            self.children.retain(|x| x.path != relation_path);
            self.children
                .retain(|x| !x.path.starts_with(&relation_path));
        } else {
            self.children.retain(|x| x.path != relation_path);
        }
    }

    // 排序
    pub fn sort(&mut self) -> Folder {
        let mut tree_map: BTreeMap<(String, bool), Item> = BTreeMap::new();
        for child in self.children.clone() {
            tree_map.insert((child.path.clone(), child.is_dir), child);
        }
        self.children = tree_map.values().cloned().collect();
        self.clone()
    }
}
