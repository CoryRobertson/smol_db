use crate::handle_client::handle_client;
use futures::executor::ThreadPool;
use futures::task::SpawnExt;
use smol_db_common::prelude::DBList;
#[cfg(feature = "logging")]
use smol_db_common::{
    logging::log_entry::LogEntry, logging::log_level::LogLevel, logging::log_message::LogMessage,
    logging::logger::Logger,
};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

#[tracing::instrument(skip(logger))]
pub(crate) async fn user_listener(
    listener: TcpListener,
    #[cfg(feature = "logging")] logger: Arc<Logger>,
    db_list: Arc<RwLock<DBList>>,
    thread_pool: &ThreadPool,
) {
    for income in listener.incoming() {
        let stream = income.expect("Failed to receive tcp stream");

        #[cfg(feature = "logging")]
        let msg = {
            stream
                .peer_addr()
                .map(|socket| format!("{}", socket))
                .map_err(|err| format!("{:?}", err))
                .unwrap_or_else(|s| s)
        };

        #[cfg(feature = "logging")]
        let _ = logger.log(&LogEntry::new(
            LogMessage::new(format!("New client connected: {}", msg).as_str()),
            LogLevel::Info,
        ));

        let client_future = handle_client(
            stream,
            db_list.clone(),
            #[cfg(feature = "logging")]
            logger.clone(),
        );

        let spawn_res = thread_pool.spawn(client_future);

        #[cfg(debug_assertions)]
        println!("{:?}", spawn_res);
    }
}
