use crate::db_data::DBData;
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
    /// Packet used in establishing end to end encryption, requests the server to store the sent public key
    PubKey(RsaPublicKey),
    /// Request the server to setup end to end encryption
    SetupEncryption,
}

impl DBPacket {
    #[cfg(feature = "statistics")]
    pub fn new_get_stats(dbname: &str) -> Self {
        Self::GetStats(DBPacketInfo::new(dbname))
    }

    /// Creates a new Read `DBPacket` from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> Self {
        Self::Read(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    /// Creates a new Delete Data `DBPacket`. This packet when sent to the server requests the server to delete the given location in the given database name.
    pub fn new_delete_data(dbname: &str, location: &str) -> Self {
        Self::DeleteData(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    /// Creates a new `GetRole` `DBPacket`, this packet when sent to the server will request the server to respond with the role of the given client.
    pub fn new_get_role(dbname: &str) -> Self {
        Self::GetRole(DBPacketInfo::new(dbname))
    }

    /// Creates a new `GetDBSettings` packet, this packet when sent to the server will request the db settings of a database, requires super admin privileges.
    pub fn new_get_db_settings(dbname: &str) -> Self {
        Self::GetDBSettings(DBPacketInfo::new(dbname))
    }

    /// Creates a new `SetDBSettings` packet which when sent to the server, will change the db settings of a database, requires super admin privileges.
    pub fn new_set_db_settings(dbname: &str, db_settings: DBSettings) -> Self {
        Self::ChangeDBSettings(DBPacketInfo::new(dbname), db_settings)
    }

    /// Creates a new `SetKey` `DBPacket` from a key. This represents the users key which determines their permissions on the server.
    /// This packet when sent to the server will set the key of the client regarding its permission status.
    pub const fn new_set_key(key: String) -> Self {
        Self::SetKey(key)
    }

    /// Creates a new Write `DBPacket` from a name of a database and location string to write to.
    /// This packet when sent to the server will request to write the data to the given location, requires permissions to operate potentially.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> Self {
        Self::Write(
            DBPacketInfo::new(dbname),
            DBLocation::new(location),
            DBData::new(data.to_string()),
        )
    }

    /// Creates a new `CreateDB` `DBPacket` from a name of a database.
    /// Creates a DB on the server with the given name and settings, requires super admin privileges.
    pub fn new_create_db(dbname: &str, db_settings: DBSettings) -> Self {
        Self::CreateDB(DBPacketInfo::new(dbname), db_settings)
    }

    /// Creates a new `DeleteDB` `DBPacket` from a name of a database.
    /// Deletes the given db from the server, requires super admin privileges.
    pub fn new_delete_db(dbname: &str) -> Self {
        Self::DeleteDB(DBPacketInfo::new(dbname))
    }

    /// Creates a `ListDB` packet.
    /// When sent to the server, lists the databases contained on the server
    pub const fn new_list_db() -> Self {
        Self::ListDB
    }

    /// Creates a `ListDBContents` packet
    /// When sent to the server, lists the contents of a given db, requires permission to do so, which depends on the given database.
    pub fn new_list_db_contents(db_name: &str) -> Self {
        Self::ListDBContents(DBPacketInfo::new(db_name))
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
