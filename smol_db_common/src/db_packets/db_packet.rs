use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
pub enum DBPacket<T> {
    /// Read(db to operate on, key to read the db using)
    Read(DBPacketInfo, DBLocation),
    /// Write(db to operate on, key to write to the db using, data to write to the key location)
    Write(DBPacketInfo, DBLocation, DBData<T>),
    /// CreateDB(db to create)
    CreateDB(DBPacketInfo),
    /// DeleteDB(db to delete)
    DeleteDB(DBPacketInfo),
    //TODO: ListDB packet type? probably has no information inside since it will need to be non-specific?

    //TODO: ListContents of db packet type too maybe? returns the entire hashmap serialized?
}

impl<T> DBPacket<T>
    where for<'a> T: Serialize + Deserialize<'a>,
{
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket<T> {
        DBPacket::Read(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: T) -> DBPacket<T> {
        DBPacket::Write(
            DBPacketInfo::new(dbname),
            DBLocation::new(location),
            DBData::new(data),
        )
    }

    /// Creates a new CreateDB DBPacket from a name of a database.
    pub fn new_create_db(dbname: &str) -> DBPacket<T> {
        DBPacket::CreateDB(DBPacketInfo::new(dbname))
    }

    /// Creates a new DeleteDB DBPacket from a name of a database.
    pub fn new_delete_db(dbname: &str) -> DBPacket<T> {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    /// Serializes a DBPacket into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn deserialize_packet(buf: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(buf)
    }
}
