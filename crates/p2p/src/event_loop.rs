use crate::{
    behaviour::BehaviourEvent,
    constant::BLOCK_PROTOCOL,
    error::P2PError,
    peer::{Latency, Peer, PeerConnectionCandidate},
    Behaviour, Event, Node,
};
use libp2p::futures::AsyncWriteExt;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{ConnectionError, DialError};
use libp2p::{
    futures::StreamExt, kad, mdns, ping, swarm::SwarmEvent, Multiaddr, PeerId, Stream, Swarm,
};
use p2p_block::message::Message;
use p2p_block::{StreamData, Transfer};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::PoisonError;
use std::{collections::HashMap, io, sync::Arc};
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct FilePath {
    pub hash: String,
    pub file_path: String,
}

pub struct EventLoop<T: StreamData + core::fmt::Debug> {
    node: Arc<Node<T>>,
    swarm: Swarm<Behaviour>,
    relay_rx: Receiver<Multiaddr>,
    // list of need reconnect
    reconnect_list: Arc<tokio::sync::RwLock<HashSet<PeerId>>>,
}

impl<T: StreamData + core::fmt::Debug + 'static> EventLoop<T> {
    pub fn new(node: Arc<Node<T>>, swarm: Swarm<Behaviour>, relay_rx: Receiver<Multiaddr>) -> Self {
        let reconnect_list: Arc<tokio::sync::RwLock<HashSet<PeerId>>> =
            Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        Self {
            node,
            swarm,
            relay_rx,
            reconnect_list,
        }
    }

    pub async fn spawn(&mut self) -> Result<(), P2PError<io::Error>> {
        // let mut swarm = build_swarm(self.identity.clone(), self.metadata()).unwrap();
        tracing::info!("Local peer id: {:?}", self.swarm.local_peer_id());
        // listen
        // swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // 添加公网peer
        // swarm
        //     .behaviour_mut()
        //     .auto_nat
        //     .add_server(self.server_peer_id, Some(self.server_address.clone()));

        let control = self.node.control.clone();

        let mut incoming_streams = control.clone().accept(BLOCK_PROTOCOL)?;

        let (reconnect_tx, mut reconnect_rx) = mpsc::channel::<PeerId>(10);
        // start the thread of reconnect
        self.reconnect_thread(reconnect_tx);

        loop {
            tokio::select! {
                // receive peer_id what needs reconnect from reconnect thread
                Some(peer) = reconnect_rx.recv() => self.handle_reconnect(peer),
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                Some((peer, stream)) = incoming_streams.next() => self.handle_message(stream, peer).await,
                Some(relay_address) = self.relay_rx.recv() => self.handle_add_relay(relay_address),
            }
        }
    }

    fn handle_add_relay(&mut self, address: Multiaddr) {
        tracing::info!("Add relay server: {address:?}");
        self.swarm
            .listen_on(address.with(Protocol::P2pCircuit))
            .expect("Listen on relay address.");
    }

    fn handle_reconnect(&mut self, peer_id: PeerId) {
        tracing::info!("Reconnecting to {peer_id:?}");

        let dial_opts: DialOpts = match self.node.get_peer_dial_opts(peer_id) {
            Ok(e) => e.into(),
            Err(e) => {
                tracing::error!("get_peer_dial_opts error: {e:?}");
                DialOpts::from(peer_id)
            }
        };
        match self.swarm.dial(dial_opts) {
            Ok(_) => {
                tracing::info!("Dialing to {peer_id:?}");
            }
            Err(e) => {
                // do nothing, when reconnect failed
                // because the reconnect thread will send the peer_id again
                tracing::error!("Dial to {peer_id:?} failed {e:?}");
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                // check if the peer is in the reconnect list
                // if it is, remove it from the list, and the connection established caused by the reconnect
                let is_reconnect = self.reconnect_list.read().await.contains(&peer_id);
                if is_reconnect {
                    // will remove peer from reconnect list, when reconnect success
                    self.reconnect_list.write().await.remove(&peer_id);
                    tracing::info!("Reconnect to {peer_id:?} success");
                } else {
                    tracing::info!("Connection established to {peer_id:?}");
                }
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                tracing::error!("Outgoing connection with {peer_id:?} error: {error:?}");
                // remove reconnect peer in some cases
                match error {
                    DialError::LocalPeerId { .. } => {}
                    DialError::NoAddresses => {}
                    DialError::DialPeerConditionFalse(_) => {}
                    DialError::Aborted => {}
                    DialError::WrongPeerId { .. } => {}
                    DialError::Denied { .. } => {}
                    DialError::Transport(_) => {}
                }
            }
            // ping 网络连接测试
            SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => {
                match event {
                    ping::Event {
                        peer,
                        result: Ok(rtt),
                        ..
                    } => {
                        // let mut peers = self.peers.write().unwrap();
                        for peer_info in self.node.peers.write().unwrap().values_mut() {
                            if peer_info.peer_id == peer {
                                // Update state with the received rtt (if available)
                                let _ = peer_info.update_state(
                                    None,
                                    Some(true),
                                    Some(Latency::Latency(rtt.as_millis() as u64)),
                                );
                                // tracing::debug!("ping update: {:#?}", peer_info);
                                break; // Exit the loop after updating the matching peer
                            }
                        }
                    }
                    ping::Event {
                        peer,
                        result: Err(ping::Failure::Timeout),
                        ..
                    } => {
                        // 超时 更新延迟时间
                        let mut peers = self.node.peers.write().unwrap();
                        if let Some(peer) = peers.get_mut(&peer) {
                            let _ = peer.update_state(None, Some(false), Some(Latency::Timeout));
                        }
                        tracing::error!("ping: timeout to {}", peer.to_base58());
                    }
                    ping::Event {
                        peer,
                        result: Err(ping::Failure::Unsupported),
                        ..
                    } => {
                        // 不支持ping协议
                        // 更新延迟时间
                        let mut peers = self.node.peers.write().unwrap();
                        if let Some(peer) = peers.get_mut(&peer) {
                            let _ = peer.update_state(None, Some(false), Some(Latency::Timeout));
                        }
                        tracing::error!(
                            "ping: {} does not support ping protocol",
                            peer.to_base58()
                        );
                    }
                    ping::Event {
                        peer,
                        result: Err(ping::Failure::Other { error }),
                        ..
                    } => {
                        // 其他错误
                        // 更新延迟时间
                        let mut peers = self.node.peers.write().unwrap();
                        if let Some(peer) = peers.get_mut(&peer) {
                            let _ = peer.update_state(None, Some(false), Some(Latency::Timeout));
                        }
                        tracing::error!("ping: ping::Failure with {}: {error}", peer.to_base58());
                    }
                }
            }

            // 内网发现
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    if let Ok(mut peers) = self.node.peers.write() {
                        match peers.entry(peer_id) {
                            std::collections::hash_map::Entry::Occupied(_) => {}
                            std::collections::hash_map::Entry::Vacant(e) => {
                                let peer = Peer::new(peer_id, multiaddr.clone());
                                let _ = peer.update_state(
                                    Some(PeerConnectionCandidate::SocketAddr(multiaddr.clone())),
                                    Some(false),
                                    Some(Latency::Timeout),
                                );
                                tracing::debug!("mDNS discover peer: {peer:#?}");
                                e.insert(Arc::new(peer));
                            }
                        }
                    }

                    // add known address to kademlia
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddr.clone());

                    match self.swarm.dial(peer_id) {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!("Dial {peer_id} error: {e:?}");
                        }
                    }

                    tracing::info!("connect {peer_id:#?},");
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                tracing::info!("mDNS discover peer has expired: {list:?}");
            }

            // metadata
            SwarmEvent::Behaviour(BehaviourEvent::Metadata(p2p_metadata::Event::Sent {
                peer_id,
                ..
            })) => {
                tracing::debug!("Sent metadata info to {peer_id:?}")
            }

            SwarmEvent::Behaviour(BehaviourEvent::Metadata(p2p_metadata::Event::Received {
                peer_id,
                info,
            })) => {
                tracing::debug!("Received metadata info from {peer_id:?}, {info:#?}");
                if let Ok(mut peers) = self.node.peers.write() {
                    match peers.entry(peer_id) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            let peer = entry.get_mut();
                            let metadata: HashMap<String, String> = info.into();
                            let _ = peer.metadata_mut().extend(metadata);
                        }
                        std::collections::hash_map::Entry::Vacant(_) => {}
                    }
                }
            }
            // HDT
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed { result, .. },
            )) => match result {
                kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                    key,
                    providers,
                    ..
                })) => {
                    for peer in providers {
                        println!(
                            "Peer {peer:?} provides key {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                }
                kad::QueryResult::GetProviders(Err(err)) => {
                    eprintln!("Failed to get providers: {err:?}");
                }
                kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(
                    kad::PeerRecord {
                        record: kad::Record { key, value, .. },
                        ..
                    },
                ))) => {
                    println!(
                        "Got record {:?} {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap(),
                        std::str::from_utf8(&value).unwrap(),
                    );
                }
                kad::QueryResult::GetRecord(Ok(_)) => {}
                kad::QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {err:?}");
                }
                kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                    println!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                kad::QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {err:?}");
                }
                kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                    println!(
                        "Successfully put provider record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                kad::QueryResult::StartProviding(Err(err)) => {
                    eprintln!("Failed to put provider record: {err:?}");
                }
                _ => {}
            },

            // autonat
            // SwarmEvent::Behaviour(BehaviourEvent::AutoNat(event)) => {
            //     match event {
            //         autonat::Event::InboundProbe(InboundProbeEvent::Request{peer,..}) => {
            //             tracing::debug!("autonat InboundProbe Request: {peer:?}")
            //         },
            //         autonat::Event::InboundProbe(InboundProbeEvent::Response{peer,..}) => {
            //             tracing::debug!("autonat InboundProbe Response: {peer:?}")
            //         },
            //         autonat::Event::OutboundProbe(OutboundProbeEvent::Request{peer,..}) => {
            //             tracing::debug!("autonat OutboundProbe Request: {peer:?}")
            //         },
            //         autonat::Event::OutboundProbe(OutboundProbeEvent::Response{peer,..}) => {
            //             tracing::debug!("autonat OutboundProbe Response: {peer:?}")
            //         },
            //         e => tracing::debug!("autonat other event {e:?}"),
            //     }
            // }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("Local node is listening on {address}");
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                tracing::warn!("ConnectionClosed: {peer_id:?}, {cause:?}");

                if let Ok(mut peers) = self.node.peers.write() {
                    peers.remove(&peer_id);
                } else {
                    tracing::error!("ConnectionClosed: can not get write from node.peers");
                }

                if let Some(cause) = cause {
                    match cause {
                        ConnectionError::IO(_) => {
                            self.reconnect_list.write().await.insert(peer_id);
                        }
                        ConnectionError::KeepAliveTimeout => {
                            self.reconnect_list.write().await.insert(peer_id);
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(event) => tracing::info!("event: {event:?}"),
            e => {
                tracing::debug!("Uncaught swarm event: {e:?}")
            }
        }
    }

    async fn handle_message(&mut self, mut stream: Stream, peer: PeerId) {
        let message_res: Result<Message<T>, io::Error> = Message::from_stream(&mut stream).await;
        match message_res {
            Ok(message) => {
                match message {
                    Message::Share(share) => {
                        tracing::info!("Received share request: {:?}", share);
                        // todo move to receiver
                        let id = share.id;
                        let (tx, rx) = oneshot::channel::<Option<Vec<PathBuf>>>();
                        tracing::info!(
                            "({id}): received file from peer '{}' with block size '{:?}' with info '{:?}'",
                            peer,
                            share.block_size,
                            share
                        );

                        // 保存配对请求
                        self.node
                            .share_pairing_reqs
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .insert(id, tx);
                        self.node
                            .share_payloads
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .insert(id, share.clone());

                        let event = Event::ShareRequest {
                            id,
                            peer_id: peer,
                            peer_name: self.node.hostname(),
                            file_list: share.file_list.clone(),
                            share_info: share.info.clone(),
                        };

                        tracing::info!("({id}): sending share request event : {:?}", event);

                        // 发送请求
                        let _ = self.node.events.send(event);

                        tracing::info!("({id}): waiting for response");

                        // 接收请求
                        tokio::select! {
                            file_path_list = rx => {
                                match file_path_list {
                                    Ok(Some(file_path_list)) => {
                                        if file_path_list.len() != share.file_list.len() {
                                            tracing::error!("({id}): do not provider enough path to receive data");
                                            return;
                                        }

                                        tracing::info!("({id}): accepted share request, {file_path_list:?}");

                                        let cancelled = Arc::new(AtomicBool::new(false));

                                        self.node.share_cancellations
                                            .lock()
                                            .unwrap_or_else(PoisonError::into_inner)
                                            .insert(id, cancelled.clone());

                                        stream.write_all(&[1]).await.map_err(|err| {
                                            tracing::error!("({id}): error sending continuation bit: '{err:?}'");

                                            // TODO: Send error to the frontend

                                            // TODO: make sure the other peer times out or we retry???
                                        }).unwrap();

                                        // 百分比
                                        let mut transfer = Transfer::new(&share, |percent| {
                                            self.node.events.send(Event::ShareProgress {
                                                id,
                                                percent,
                                                // files: hashes_clone.clone()
                                                // file_path: file_path.to_string_lossy().to_string()
                                                share_info: share.info.clone()
                                            }).ok();
                                        }, &cancelled);

                                        // 前面已经判断过路径数量保持一致了
                                        for i in 0..share.file_list.len() {
                                            let target_file_path = file_path_list[i].clone();
                                            match File::create(&target_file_path).await {
                                                Ok(f) => {
                                                    tracing::info!("({id}): create file at '{target_file_path:?}'");
                                                    let f: BufWriter<File> = BufWriter::new(f);
                                                    if let Err(err) = transfer.receive(&mut stream, f).await {
                                                        tracing::error!("({id}): error receiving file: '{err:?}'");
                                                        // TODO: Send error to frontend
                                                    }
                                                }
                                                Err(err) => {
                                                    tracing::error!("({id}): error creating file at '{target_file_path:?}': '{err:?}'");
                                                }
                                            }
                                        }

                                        tracing::info!("({id}): complete");
                                    },
                                    Ok(None) => {
                                        tracing::info!("({id}): rejected");

                                        stream.write_all(&[0]).await.map_err(|err| {
                                            tracing::error!("({id}): error sending rejection: '{err:?}'");
                                        }).unwrap();

                                        stream.flush().await.map_err(|err| {
                                            tracing::error!("({id}): error flushing rejection: '{err:?}'");
                                        }).unwrap();
                                    }
                                    Err(_) => {
                                        tracing::warn!("({id}): error with Spacedrop pairing request receiver!");
                                    }
                                }
                            }
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                                // 60s 超时
                                tracing::info!("({id}): timeout, rejecting!");
                                stream.write_all(&[0]).await.map_err(|err| {
                                    tracing::error!("({id}): error reject bit: '{err:?}'");
                                }).unwrap();
                                stream.flush().await.map_err(|err| {
                                    tracing::error!("({id}): error flushing reject bit: '{err:?}'");
                                }).unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("p2p incoming_streams err: {e}")
            }
        }
    }

    /// Reconnect thread
    /// just for bad network
    /// not support when the peer is changed
    fn reconnect_thread(&self, tx: mpsc::Sender<PeerId>) {
        let reconnect_list = self.reconnect_list.clone();
        tokio::spawn(async move {
            loop {
                // iters the reconnect list every 10 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let reconnect_read = reconnect_list.read().await;
                // clone to release the read lock
                let reconnect_clone = reconnect_read.clone();
                for peer_id in reconnect_clone.iter() {
                    let _ = tx.send(*peer_id).await;
                    tracing::info!("Send reconnect event with {peer_id:?}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });
    }
}
