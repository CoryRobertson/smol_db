//! Common library between the client and server for `smol_db`

pub mod db;
pub mod statistics;
pub mod db_content;
pub mod db_data;
pub mod db_list;
pub mod db_packets;
/// Public exposing of the `simple_logger_rs` logging library for use from dependants on `smol_db_common`
pub use simple_logger_rs::logging;
