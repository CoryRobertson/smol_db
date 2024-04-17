use crate::handle_client::handle_client;
use futures::executor::ThreadPool;
use futures::task::SpawnExt;
use smol_db_common::prelude::DBList;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

#[tracing::instrument(skip(db_list))]
pub async fn user_listener(
    listener: TcpListener,
    db_list: Arc<RwLock<DBList>>,
    thread_pool: &ThreadPool,
) {
    info!("Listening for users");
    for income in listener.incoming() {
        let stream = income.expect("Failed to receive tcp stream");

        info!(
            "New client connected: {}",
            stream
                .peer_addr()
                .map(|socket| format!("{}", socket))
                .map_err(|err| format!("{:?}", err))
                .unwrap_or_else(|s| s)
        );

        let client_future = handle_client(stream, db_list.clone());

        let spawn_res = thread_pool.spawn(client_future);

        debug!("Spawned client in thread pool: {:?}", spawn_res);
    }
}
