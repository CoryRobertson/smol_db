//! Binary application that runs a `smol_db` server instance
#[cfg(not(feature = "no-saving"))]
use crate::cache_invalidator::cache_invalidator;
use crate::new_user_handler::user_listener;
use futures::executor::ThreadPoolBuilder;
use futures::join;
use smol_db_common::db_list::DBList;
#[cfg(not(feature = "no-saving"))]
use std::fs;
use std::net::TcpListener;
use std::process::exit;
use std::sync::{Arc, RwLock};
use tracing::info;
#[cfg(feature = "tracing")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(not(feature = "no-saving"))]
mod cache_invalidator;
mod handle_client;
mod new_user_handler;

type DBListThreadSafe = Arc<RwLock<DBList>>;

#[allow(dead_code)]
const LOG_FILE_PATH: &str = "./data/log.log";

fn main() {
    #[cfg(feature = "tracing")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .expect("setup tracy layer");

    #[cfg(not(feature = "tracing"))]
    let _ = tracing_subscriber::fmt::try_init();

    let listener = TcpListener::bind("0.0.0.0:8222").expect("Failed to bind to port 8222.");

    let thread_pool = ThreadPoolBuilder::new()
        .name_prefix("[Smol_DB]")
        .create()
        .unwrap();

    {
        print!("Features enabled:");
        #[cfg(feature = "tracing")]
        print!(" Tracing");
        #[cfg(feature = "statistics")]
        print!(" Statistics");
        #[cfg(feature = "no-saving")]
        print!(" No-Saving");
        println!();
    }

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

    #[cfg(not(feature = "no-saving"))]
    let _ = fs::create_dir("./data");

    #[cfg(not(feature = "no-saving"))]
    fs::read_dir("./data").expect("Data directory ./data must exist"); // the data directory must exist, so we make sure this happens

    // control-c handler for saving things before the server shuts down.
    setup_control_c_handler(db_list.clone());

    // thread that continuously checks if caches need to be removed from cache when they get old.
    #[cfg(not(feature = "no-saving"))]
    let cache_invalidator_future = cache_invalidator(db_list.clone());

    #[cfg(feature = "no-saving")]
    let cache_invalidator_future = async {};

    let user_listener = user_listener(listener, db_list.clone(), &thread_pool);

    info!("Waiting for connections on port 8222");

    futures::executor::block_on(async {
        join!(cache_invalidator_future, user_listener,);
    });
}

#[tracing::instrument]
fn setup_control_c_handler(db_list: DBListThreadSafe) {
    ctrlc::set_handler(move || {
        info!("Received CTRL+C, gracefully shutting down program.");
        let lock = db_list.read().unwrap();
        info!("{:?}", lock.list.read().unwrap());

        #[cfg(not(feature = "no-saving"))]
        {
            lock.save_db_list();
            lock.save_all_db();
            info!("Saved all db files and db list.");
        }
        info!("Saved all db files and db list.");
        exit(0);
    })
    .unwrap();
}
