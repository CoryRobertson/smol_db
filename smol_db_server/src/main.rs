//! Binary application that runs a `smol_db` server instance
#[cfg(not(feature = "no-saving"))]
use crate::cache_invalidator::cache_invalidator;
use crate::new_user_handler::user_listener;
use futures::executor::ThreadPoolBuilder;
use futures::join;
use smol_db_common::db_list::DBList;
#[cfg(feature = "logging")]
use smol_db_common::{
    logging::log_entry::LogEntry, logging::log_level::LogLevel, logging::log_message::LogMessage,
    logging::logger::Logger,
};
#[cfg(not(feature = "no-saving"))]
use std::fs;
use std::net::TcpListener;
#[cfg(feature = "logging")]
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
#[cfg(feature = "tracing")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(not(feature = "no-saving"))]
mod cache_invalidator;
mod handle_client;
mod new_user_handler;

type DBListThreadSafe = Arc<RwLock<DBList>>;

#[cfg(feature = "logging")]
const LOG_FILE_PATH: &str = "./data/log.log";


fn main() {

    #[cfg(feature = "tracing")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default())
    ).expect("setup tracy layer");

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
        #[cfg(feature = "logging")]
        print!(" Logging");
        #[cfg(feature = "no-saving")]
        print!(" No-Saving");
        println!();
    }

    #[cfg(feature = "logging")]
    let logger = Arc::new(Logger::new(PathBuf::from(LOG_FILE_PATH)).unwrap());

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

    #[cfg(not(feature = "no-saving"))]
    let _ = fs::create_dir("./data");

    #[cfg(not(feature = "no-saving"))]
    fs::read_dir("./data").expect("Data directory ./data must exist"); // the data directory must exist, so we make sure this happens

    // control-c handler for saving things before the server shuts down.
    setup_control_c_handler(
        #[cfg(feature = "logging")]
        logger.clone(),
        db_list.clone(),
    );

    // thread that continuously checks if caches need to be removed from cache when they get old.
    #[cfg(not(feature = "no-saving"))]
    let cache_invalidator_future = cache_invalidator(
        #[cfg(feature = "logging")]
        logger.clone(),
        db_list.clone(),
    );

    #[cfg(feature = "no-saving")]
    let cache_invalidator_future = async {};

    let user_listener = user_listener(
        listener,
        #[cfg(feature = "logging")]
        logger.clone(),
        db_list.clone(),
        &thread_pool,
    );

    println!("Waiting for connections on port 8222");

    futures::executor::block_on(async {
        join!(cache_invalidator_future, user_listener,);
    });
}

#[tracing::instrument(skip(logger))]
fn setup_control_c_handler(
    #[cfg(feature = "logging")] logger: Arc<Logger>,
    db_list: DBListThreadSafe,
) {
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, gracefully shutting down program.");
        #[cfg(feature = "logging")]
        let _ = logger.log(&LogEntry::new(
            LogMessage::new("Received CTRL+C, gracefully shutting down program."),
            LogLevel::Info,
        ));
        let lock = db_list.read().unwrap();
        println!("{:?}", lock.list.read().unwrap());

        #[cfg(not(feature = "no-saving"))]
        {
            lock.save_db_list();
            lock.save_all_db();
            println!("Saved all db files and db list.");
        }
        #[cfg(feature = "logging")]
        logger
            .log(&LogEntry::new(
                LogMessage::new("Saved all db files and db list."),
                LogLevel::Info,
            ))
            .expect("Failed to log saving message to log file");
        exit(0);
    })
    .unwrap();
}
