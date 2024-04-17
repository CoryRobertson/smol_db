use crate::db_data::DBData;
use crate::db_packets::db_keyed_list_location::DBKeyedListLocation;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_settings::DBSettings;
use crate::encryption::encrypted_data::EncryptedData;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
/// This enum will get breaking changes until **git rev** `1c81904f00a69025aad49091abe3d56fd45e1144` can be fixed, until then, unsure how to avoid it.
/// Workaround: use an exhaustive pattern match system e.g.:
/// ```rust
/// use smol_db_common::db_packets::db_packet::DBPacket;
///
/// let p = DBPacket::ListDB;
/// match p {
/// DBPacket::ListDB => {}
///  _ => {} // this line is needed to not have breaking changes
/// }
/// ```
pub enum DBPacket {
    /// Read(db to operate on, key to read the db using)
    Read(DBPacketInfo, DBLocation),
    /// Write(db to operate on, key to write to the db using, data to write to the key location)
    Write(DBPacketInfo, DBLocation, DBData),
    /// DeleteData(db to operate on, key to delete data from)
    DeleteData(DBPacketInfo, DBLocation),
    /// CreateDB(db to create)
    CreateDB(DBPacketInfo, DBSettings),
    /// DeleteDB(db to delete)
    DeleteDB(DBPacketInfo),
    /// ListDB
    ListDB,
    /// ListDBContents(db to read from)
    ListDBContents(DBPacketInfo),
    /// Adds an admin to the database with the given hash
    AddAdmin(DBPacketInfo, String),
    /// Adds a user to the database with the given hash
    AddUser(DBPacketInfo, String),
    /// Sets the clients key to the given hash
    SetKey(String),
    /// Returns the DBSettings struct within the given db
    GetDBSettings(DBPacketInfo),
    /// Sets the DBSettings struct within the given db to the new settings struct.
    ChangeDBSettings(DBPacketInfo, DBSettings),
    /// GetRole(db to read role from)
    GetRole(DBPacketInfo),
    /// GetStats gets the statistics object if the feature is compiled
    GetStats(DBPacketInfo),
    /// Encrypted packet, used to allow the server to identify when data needs to be decrypted
    Encrypted(EncryptedData),
    /// Packet used in establishing end-to-end encryption, requests the server to store the sent public key
    PubKey(RsaPublicKey),
    /// Request the server to set up end-to-end encryption
    SetupEncryption,
    /// Request the server to begin streaming values from a given DB to the user
    StreamReadDb(DBPacketInfo),
    /// Request the next item in the stream, if one is open
    ReadyForNextItem,
    /// Tell the server that the client wants to stop streaming values from a DB
    EndStreamRead,

    /// Add data to a named list in a database
    AddToList(DBPacketInfo, DBKeyedListLocation, DBData),
    /// Read data from a named list in a database
    ReadFromList(DBPacketInfo, DBKeyedListLocation),
    /// Remove data from a named list in a database
    RemoveFromList(DBPacketInfo, DBKeyedListLocation),
    /// Stream in order starting from the given index a named list in a database
    StreamList(DBPacketInfo, DBKeyedListLocation),
    /// Request the length of a list in a database if it exists
    GetListLength(DBPacketInfo, DBKeyedListLocation),
    /// Clear all data from a list in a database if it exists
    ClearList(DBPacketInfo, DBKeyedListLocation),
}

impl DBPacket {
    #[must_use]
    pub fn new_get_list_length(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<DBKeyedListLocation>,
    ) -> Self {
        Self::GetListLength(table_name.into(), list_name.into())
    }

    #[must_use]
    pub fn new_clear_list(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<DBKeyedListLocation>,
    ) -> Self {
        Self::ClearList(table_name.into(), list_name.into())
    }

    #[must_use]
    pub fn new_stream_table(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::StreamReadDb(dbname.into())
    }

    #[must_use]
    pub fn new_stream_db_list(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<String>,
        start_idx: Option<usize>,
    ) -> Self {
        Self::StreamList(
            table_name.into(),
            DBKeyedListLocation::new(start_idx, list_name.into()),
        )
    }

    #[must_use]
    pub fn new_add_db_list(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<String>,
        start_idx: Option<usize>,
        data: impl Into<DBData>,
    ) -> Self {
        Self::AddToList(
            table_name.into(),
            DBKeyedListLocation::new(start_idx, list_name.into()),
            data.into(),
        )
    }

    #[must_use]
    pub fn new_read_from_db_list(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<String>,
        start_idx: usize,
    ) -> Self {
        Self::ReadFromList(
            table_name.into(),
            DBKeyedListLocation::new(Some(start_idx), list_name.into()),
        )
    }

    #[must_use]
    pub fn new_remove_from_db_list(
        table_name: impl Into<DBPacketInfo>,
        list_name: impl Into<String>,
        start_idx: Option<usize>,
    ) -> Self {
        Self::RemoveFromList(
            table_name.into(),
            DBKeyedListLocation::new(start_idx, list_name.into()),
        )
    }

    #[cfg(feature = "statistics")]
    #[must_use]
    pub fn new_get_stats(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::GetStats(dbname.into())
    }

    /// Creates a new Read `DBPacket` from a name of a database and location string to read from.
    #[must_use]
    pub fn new_read(dbname: impl Into<DBPacketInfo>, location: impl Into<DBLocation>) -> Self {
        Self::Read(dbname.into(), location.into())
    }

    /// Creates a new Delete Data `DBPacket`. This packet when sent to the server requests the server to delete the given location in the given database name.
    #[must_use]
    pub fn new_delete_data(
        dbname: impl Into<DBPacketInfo>,
        location: impl Into<DBLocation>,
    ) -> Self {
        Self::DeleteData(dbname.into(), location.into())
    }

    /// Creates a new `GetRole` `DBPacket`, this packet when sent to the server will request the server to respond with the role of the given client.
    #[must_use]
    pub fn new_get_role(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::GetRole(dbname.into())
    }

    /// Creates a new `GetDBSettings` packet, this packet when sent to the server will request the db settings of a database, requires super admin privileges.
    #[must_use]
    pub fn new_get_db_settings(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::GetDBSettings(dbname.into())
    }

    /// Creates a new `SetDBSettings` packet which when sent to the server, will change the db settings of a database, requires super admin privileges.
    #[must_use]
    pub fn new_set_db_settings(dbname: impl Into<DBPacketInfo>, db_settings: DBSettings) -> Self {
        Self::ChangeDBSettings(dbname.into(), db_settings)
    }

    /// Creates a new `SetKey` `DBPacket` from a key. This represents the users key which determines their permissions on the server.
    /// This packet when sent to the server will set the key of the client regarding its permission status.
    #[must_use]
    pub const fn new_set_key(key: String) -> Self {
        Self::SetKey(key)
    }

    /// Creates a new Write `DBPacket` from a name of a database and location string to write to.
    /// This packet when sent to the server will request to write the data to the given location, requires permissions to operate potentially.
    #[must_use]
    pub fn new_write(
        dbname: impl Into<DBPacketInfo>,
        location: impl Into<DBLocation>,
        data: impl Into<DBData>,
    ) -> Self {
        Self::Write(dbname.into(), location.into(), data.into())
    }

    /// Creates a new `CreateDB` `DBPacket` from a name of a database.
    /// Creates a DB on the server with the given name and settings, requires super admin privileges.
    #[must_use]
    pub fn new_create_db(dbname: impl Into<DBPacketInfo>, db_settings: DBSettings) -> Self {
        Self::CreateDB(dbname.into(), db_settings)
    }

    /// Creates a new `DeleteDB` `DBPacket` from a name of a database.
    /// Deletes the given db from the server, requires super admin privileges.
    #[must_use]
    pub fn new_delete_db(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::DeleteDB(dbname.into())
    }

    /// Creates a `ListDB` packet.
    /// When sent to the server, lists the databases contained on the server
    #[must_use]
    pub const fn new_list_db() -> Self {
        Self::ListDB
    }

    /// Creates a `ListDBContents` packet
    /// When sent to the server, lists the contents of a given db, requires permission to do so, which depends on the given database.
    #[must_use]
    pub fn new_list_db_contents(dbname: impl Into<DBPacketInfo>) -> Self {
        Self::ListDBContents(dbname.into())
    }

    /// Serializes a `DBPacket` into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    /// Deserialize a `DBPacket` from a buf.
    pub fn deserialize_packet(buf: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(buf)
    }
}
