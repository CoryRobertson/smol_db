//! Library containing the structs that manage the client to connect to `smol_db`

mod client;
pub mod client_error;
mod table_iter;
pub use smol_db_common::{
    db::Role, db_packets::db_packet_response::DBPacketResponseError,
    db_packets::db_packet_response::DBSuccessResponse, db_packets::db_settings,
};

/// Easy usable module containing everything needed to use the client library normally
pub mod prelude {
    pub use crate::client::SmolDbClient;
    pub use crate::client_error;
    pub use crate::client_error::ClientError::DBResponseError;
    pub use crate::table_iter::TableIter;
    pub use smol_db_common::db::Role;
    pub use smol_db_common::db::Role::*;
    pub use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    pub use smol_db_common::db_packets::db_packet_response::DBPacketResponseError::*;
    pub use smol_db_common::db_packets::db_packet_response::DBSuccessResponse;
    pub use smol_db_common::db_packets::db_packet_response::DBSuccessResponse::SuccessNoData;
    pub use smol_db_common::db_packets::db_packet_response::DBSuccessResponse::SuccessReply;
    pub use smol_db_common::db_packets::db_settings::DBSettings;
    #[cfg(feature = "statistics")]
    pub use smol_db_common::statistics::DBStatistics;
}
