use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Debug,Clone, Hash, PartialEq, Eq,PartialOrd,Ord)]
/// A struct that describes the name of a database to be searched through.
pub struct DBPacketInfo {
    dbname: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A struct that describes a key to search with through the database.
pub struct DBLocation {
    location: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData {
    data: String
}

impl DBData {
    /// Function to create a new DBData struct for a DBPacket::Write packet.
    pub fn new(data: String) -> Self {
        // TODO: eventually revise this with some amount of error checking
        Self {
            data
        }
    }

    /// Getter function for the data inside the DBData struct.
    pub fn get_data(&self) -> &str {
        &self.data
    }
}

impl DBLocation {
    /// Function to create a new DBLocation struct from a given location.
    pub fn new(location: &str) -> Self {
        Self{ location: location.to_string() }
    }

    /// Function to retrieve the location as a key from the struct.
    pub fn as_key(&self) -> &str {
        &self.location
    }
}

impl DBPacketInfo {
    /// Function to create a new DBPacketInfo struct with the given name
    pub fn new(dbname: &str) -> Self {
        Self{ dbname: dbname.to_string() }
    }

    /// Function to retrieve the name from the DBPacketInfo struct.
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
///
/// Read(db to operate on, key to read the db using)
/// Write(db to operate on, key to write to the db using)
/// CreateDB(db to create)
/// DeleteDB(db to delete)
pub enum DBPacket {
    Read(DBPacketInfo, DBLocation),
    Write(DBPacketInfo, DBLocation, DBData),
    CreateDB(DBPacketInfo),
    DeleteDB(DBPacketInfo),
}

// TODO: write a DBPacketResponse enum that represents the types of responses that the database server can give
//  Examples include:
//  SUCCESS, meaning the response sent was successful, but needed no further information to be returned.
//  REPLY(StructContainingTheData(String?))
//  Error(SomeEnumErrorType)
//  For StructContainingTheData:
//  The struct could be a serializable struct that can be deserialized?
//  Most likely just a simple String so we can retain Serialization.
//  But could be an enum that denotes type so we dont have to guess what the type of the info is?
//  For SomeEnumErrorType:
//  Likely just a few different error types including DBNotFound, DataNotFound, anything else?

impl DBPacket {
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname),DBLocation::new(location))
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> DBPacket {
        DBPacket::Write(DBPacketInfo::new(dbname),DBLocation::new(location), DBData::new(data.to_string()))
    }

    /// Creates a new CreateDB DBPacket from a name of a database.
    pub fn new_create_db(dbname: &str) -> DBPacket {
        DBPacket::CreateDB(DBPacketInfo::new(dbname))
    }

    /// Creates a new DeleteDB DBPacket from a name of a database.
    pub fn new_delete_db(dbname: &str) -> DBPacket {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    /// Serializes a DBPacket into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// Struct denoting the content itself of a database. At the moment, is simply a hashmap.
pub struct DBContent {
    pub content: HashMap<String,String>,
}

impl DBContent {

    /// Reads serialized version of a DBContent struct from a string (read from a file most likely) into a DBContent struct itself.
    pub fn read_ser_data(data: String) -> serde_json::Result<Self> {
        serde_json::from_str(&data)
    }

    /// Reads from the db using the key, returning an optional of either the retrieved content, or nothing.
    pub fn read_from_db(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }

}

#[allow(clippy::derivable_impls)] // This lint is allowed so we can later make default not simply have the default impl
impl Default for DBContent{
    /// Returns a default empty HashMap.
    fn default() -> Self {
        Self{ content: HashMap::default() }
    }
}
