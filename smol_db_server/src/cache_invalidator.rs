use futures_time::task;
use futures_time::time::Duration;
use smol_db_common::prelude::DBList;
use std::sync::{Arc, RwLock};
use tracing::info;

#[tracing::instrument(skip_all)]
pub async fn cache_invalidator(db_list: Arc<RwLock<DBList>>) {
    info!("Cache invalidator spawned");
    loop {
        let invalidated_caches = db_list.read().unwrap().sleep_caches();

        db_list.read().unwrap().save_all_db();
        db_list.read().unwrap().save_db_list();

        if invalidated_caches > 0 {
            let number_of_caches_remaining = db_list.read().unwrap().cache.read().unwrap().len();
            info!(
                "Slept {} caches, {} caches remain in cache.",
                invalidated_caches, number_of_caches_remaining
            );
        }

        task::sleep(Duration::from_secs(10)).await;
    }
}
