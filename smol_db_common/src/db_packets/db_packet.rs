use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_settings::DBSettings;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
pub enum DBPacket {
    /// Read(db to operate on, key to read the db using)
    Read(DBPacketInfo, DBLocation),
    /// Write(db to operate on, key to write to the db using, data to write to the key location)
    Write(DBPacketInfo, DBLocation, DBData),
    /// DeleteData(db to operate on, key to delete data from)
    DeleteData(DBPacketInfo,DBLocation),
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
}

impl DBPacket {
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    pub fn new_delete_data(dbname: &str, location: &str) -> DBPacket {
        DBPacket::DeleteData(DBPacketInfo::new(dbname),DBLocation::new(location))
    }

    pub fn new_get_role(dbname: &str) -> DBPacket {
        DBPacket::GetRole(DBPacketInfo::new(dbname))
    }

    pub fn new_get_db_settings(dbname: &str) -> DBPacket {
        DBPacket::GetDBSettings(DBPacketInfo::new(dbname))
    }

    pub fn new_set_db_settings(dbname: &str, db_settings: DBSettings) -> DBPacket {
        DBPacket::ChangeDBSettings(DBPacketInfo::new(dbname), db_settings)
    }

    /// Creates a new SetKey DBPacket from a key. This represents the users key which determines their permissions on the server.
    pub fn new_set_key(key: String) -> DBPacket {
        DBPacket::SetKey(key)
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> DBPacket {
        DBPacket::Write(
            DBPacketInfo::new(dbname),
            DBLocation::new(location),
            DBData::new(data.to_string()),
        )
    }

    /// Creates a new CreateDB DBPacket from a name of a database.
    pub fn new_create_db(dbname: &str, db_settings: DBSettings) -> DBPacket {
        DBPacket::CreateDB(DBPacketInfo::new(dbname), db_settings)
    }

    /// Creates a new DeleteDB DBPacket from a name of a database.
    pub fn new_delete_db(dbname: &str) -> DBPacket {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    pub fn new_list_db() -> DBPacket {
        DBPacket::ListDB
    }

    pub fn new_list_db_contents(db_name: &str) -> DBPacket {
        DBPacket::ListDBContents(DBPacketInfo::new(db_name))
    }

    /// Serializes a DBPacket into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn deserialize_packet(buf: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(buf)
    }
}
