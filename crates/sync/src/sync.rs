use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use crate::{
    error::WrappedRepoError,
    event::Events,
    event_loop::spawn,
    storage::Storage,
    utils::{delete_document_by_id_string, get_document_by_id, str_to_document_id},
    SyncError,
};

use automerge::sync::{Message, State, SyncDoc};
use automerge_repo::{DocHandle, DocumentId, Repo, RepoError, RepoHandle, RepoId};
use p2p_block::SyncMessage;
use prisma_lib::PrismaClient;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Sync {
    repo_handle: RepoHandle,

    prisma_client: Arc<PrismaClient>,

    pub events: Arc<Events>,
    // 实现 优雅关机，可以设置为0
    // semaphore: Semaphore,
    pub states: Arc<Mutex<HashMap<(String, String), State>>>,
    // 用于保存上一条发送到peer的消息是否为空
    pub prev_has_message: Arc<Mutex<HashMap<(String, String), bool>>>,
}

impl Sync {
    #[must_use]
    pub fn new(id: Uuid, prisma_client: Arc<PrismaClient>) -> Self {
        let events = Arc::new(Events::new());
        let repo = Repo::new(
            Some(id.to_string()),
            Box::new(Storage::new(prisma_client.clone())),
        );

        let repo_handle = repo.run();

        let sync = Self {
            repo_handle,
            prisma_client,
            events: events.clone(),
            states: Default::default(),
            prev_has_message: Default::default(),
        };

        spawn(Arc::new(sync.clone()));

        sync
    }

    pub fn local_repo_id(&self) -> &RepoId {
        self.repo_handle.get_repo_id()
    }

    pub fn new_document(&self) -> DocHandle {
        self.repo_handle.new_document()
    }

    pub async fn new_document_with_id(&self, doc_id: String) -> Result<DocumentId, SyncError> {
        // todo 万一有这个id
        let document_id = str_to_document_id(doc_id.clone())?;
        let document = automerge::Automerge::new();
        let _ = self
            .prisma_client
            .sync()
            .create(doc_id, document.save(), vec![])
            .exec()
            .await?;
        Ok(document_id)
    }

    /// 优雅关机
    /// 1. 停止所有写入任务的完成
    /// 2. 调用stop方法
    /// 3. 关闭软件
    pub fn stop(&self) -> Result<(), SyncError> {
        // todo
        // 优雅关闭
        Ok(self
            .repo_handle
            .clone()
            .stop()
            .map_err(|e| SyncError::Repo(WrappedRepoError(e)))?)
    }

    pub async fn request_document(&self, doc_id: DocumentId) -> Result<DocHandle, RepoError> {
        self.repo_handle.clone().request_document(doc_id).await
    }

    pub async fn list_all_documents(&self) -> Result<Vec<DocumentId>, SyncError> {
        Ok(self
            .repo_handle
            .clone()
            .list_all()
            .await
            .map_err(|e| SyncError::Repo(WrappedRepoError(e)))?)
    }

    // 删除文档
    pub async fn delete_document(&self, doc_id: String) -> Result<(), SyncError> {
        Ok(delete_document_by_id_string(self.prisma_client.clone(), doc_id).await?)
    }

    // 查询是否有这个文档
    pub async fn has_document(&self, doc_id: DocumentId) -> Result<bool, SyncError> {
        Ok(get_document_by_id(self.prisma_client.clone(), doc_id).await?)
    }

    // 获取文档的状态 没有就创建
    pub fn get_document_state(&self, doc_id: DocumentId, peer: String) -> Result<State, SyncError> {
        let doc_id_string = doc_id.as_uuid_str();
        let state = {
            let mut states = self.states.lock().unwrap_or_else(PoisonError::into_inner);
            
            match states.get(&(doc_id_string.clone(), peer.clone())) {
              Some(state) => state.clone(),
              None => {
                let new_state = State::new();
                states.insert((doc_id_string, peer), new_state.clone());
                new_state
              }
            }
          };
        
        Ok(state)
    }

    pub fn save_document_state(
        &mut self,
        doc_id: DocumentId,
        peer: String,
        state: State,
    ) -> Result<(), SyncError> {
        self.states
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((doc_id.as_uuid_str(), peer.clone()), state);
        Ok(())
    }

    // 删除状态
    pub fn delete_document_state(&mut self, doc_id: String, peer: String) -> Result<(), SyncError> {
        self.states
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&(doc_id, peer));
        Ok(())
    }

    pub fn get_prev_has_message(
        &self,
        doc_id: DocumentId,
        peer: String,
    ) -> Result<bool, SyncError> {
        let doc_id_string = doc_id.as_uuid_str();
        let has_message = match self
            .prev_has_message
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(doc_id_string, peer.clone()))
        {
            Some(state) => state.clone(),
            None => false,
        };
        Ok(has_message)
    }

    pub fn save_prev_has_message(
        &self,
        doc_id: DocumentId,
        peer: String,
        value: bool,
    ) -> Result<(), SyncError> {
        self.prev_has_message
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((doc_id.as_uuid_str(), peer.clone()), value);
        Ok(())
    }

    pub fn delete_prev_has_message(
        &mut self,
        doc_id: String,
        peer: String,
    ) -> Result<(), SyncError> {
        self.prev_has_message
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&(doc_id, peer));
        Ok(())
    }

    /*
        同步流程
        1. 生成之后再接收
        2. 更新 state
        3. 直到2边没有同步消息为止
    */

    // 生成同步消息 下一步p2p发送过去
    pub async fn generate_sync_message(
        &self,
        doc_handle: DocHandle,
        mut state: &mut State,
    ) -> Result<Option<Message>, SyncError> {
        Ok(doc_handle.with_doc(|doc| doc.generate_sync_message(&mut state)))
    }

    // 接收同步消息
    pub async fn receive_sync_message(
        &self,
        doc_handle: DocHandle,
        mut state: &mut State,
        message: Message,
    ) -> Result<(), SyncError> {
        let _ = doc_handle.with_doc_mut(|doc| doc.receive_sync_message(&mut state, message));
        Ok(())
    }

    // 开始同步文档
    // todo 优化
    pub async fn sync_document(
        &mut self,
        doc_id: String,
        peer: String,
        stream: &mut (impl futures::io::AsyncRead
                  + futures::AsyncReadExt
                  + futures::AsyncWrite
                  + futures::AsyncWriteExt
                  + Unpin
                  + std::marker::Send),
    ) -> Result<(), SyncError> {
        let doc = str_to_document_id(doc_id.clone())?;
        let doc_handle = self.request_document(doc.clone()).await.unwrap(); // 一定有, 除非数据库出错

        // 创建一个新的同步状态
        let mut state: State = automerge::sync::State::new();

        let message = self
            .generate_sync_message(doc_handle.clone(), &mut state)
            .await?;

        tracing::debug!("sync_document 生成的消息 {:?}", message);

        if message.is_none() {
            // break;
            return Ok(());
        }

        self.save_document_state(doc.clone(), peer.clone(), state)?;

        self.save_prev_has_message(doc.clone(), peer.clone(), false)?;

        let pre_has_message = self.get_prev_has_message(doc.clone(), peer.clone())?;

        tracing::debug!("初始化 pre_has_message: {pre_has_message}");

        let sync_message = SyncMessage {
            doc_id: doc_id.clone(),
            peer_id: peer.clone(),
            message: Some(message.unwrap()),
        };
        let p2p_message = p2p_block::message::Message::<SyncMessage>::Sync(sync_message);
        let bytes = p2p_message.to_bytes();
        tracing::debug!("发送文档");
        stream.write_all(&bytes).await?;

        Ok(())
    }
}
