//! Common library between the client and server for `smol_db`

// TODO: Use encryption between packets being sent using pubkey and priv key from server to client and client to server

pub mod db;
pub mod db_content;
pub mod db_data;
pub mod db_list;
pub mod db_packets;
#[cfg(feature = "statistics")]
pub mod statistics;
/// Public exposing of the `simple_logger_rs` logging library for use from dependants on `smol_db_common`
pub use simple_logger_rs::logging;

pub mod prelude {
    pub use crate::db::Role;
    pub use crate::db::Role::{Admin, Other, SuperAdmin, User};
    pub use crate::db::DB;
    pub use crate::db_data::DBData;
    pub use crate::db_list::DBList;
    pub use crate::db_packets::db_location::DBLocation;
    pub use crate::db_packets::db_packet_info::DBPacketInfo;
    pub use crate::db_packets::db_packet_response::DBPacketResponseError::{
        DBAlreadyExists, DBNotFound, InvalidPermissions, UserNotFound, ValueNotFound,
    };
    pub use crate::db_packets::db_packet_response::DBSuccessResponse::{
        SuccessNoData, SuccessReply,
    };
    pub use crate::db_packets::db_packet_response::{DBPacketResponseError, DBSuccessResponse};
    pub use crate::db_packets::db_settings::DBSettings;
}
