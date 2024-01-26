use futures_time::task;
use futures_time::time::Duration;
use smol_db_common::prelude::DBList;
#[cfg(feature = "logging")]
use smol_db_common::{
    logging::log_entry::LogEntry, logging::log_level::LogLevel, logging::log_message::LogMessage,
    logging::logger::Logger,
};
use std::sync::{Arc, RwLock};

pub(crate) async fn cache_invalidator(
    #[cfg(feature = "logging")] logger: Arc<Logger>,
    db_list: Arc<RwLock<DBList>>,
) {
    loop {
        let invalidated_caches = db_list.read().unwrap().sleep_caches();

        db_list.read().unwrap().save_all_db();
        db_list.read().unwrap().save_db_list();

        if invalidated_caches > 0 {
            let number_of_caches_remaining = db_list.read().unwrap().cache.read().unwrap().len();
            let msg = format!(
                "Slept {} caches, {} caches remain in cache.",
                invalidated_caches, number_of_caches_remaining
            );
            println!("{}", msg);
            #[cfg(feature = "logging")]
            let _ = logger.log(&LogEntry::new(
                LogMessage::new(msg.as_str()),
                LogLevel::Info,
            ));
        }

        task::sleep(Duration::from_secs(10)).await;
    }
}
