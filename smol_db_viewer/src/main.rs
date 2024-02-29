//! `smol_db` viewer application, allows for viewing the state of a server and its databases.
#[cfg(feature = "tracing")]
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use crate::app::ApplicationState;

mod app;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    #[cfg(feature = "tracing")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default())
    ).expect("setup tracy layer");


    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "smol_db_viewer",
        native_options,
        Box::new(|cc| Box::new(ApplicationState::new(cc))),
    )
}
