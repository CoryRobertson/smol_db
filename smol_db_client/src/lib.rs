//! Library containing the structs that manage the client to connect to `smol_db`

use crate::client_error::ClientError;
use crate::ClientError::{
    BadPacket, PacketDeserializationError, PacketSerializationError, SocketReadError,
    SocketWriteError, UnableToConnect,
};
use serde::{Deserialize, Serialize};
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_info::DBPacketInfo;

use smol_db_common::db_packets::db_settings::DBSettings;
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};

pub mod client_error;
use crate::client_error::ClientError::DBResponseError;
pub use smol_db_common::db::Role;
pub use smol_db_common::db_packets::db_packet_response::DBPacketResponseError;
pub use smol_db_common::db_packets::db_packet_response::DBSuccessResponse;
pub use smol_db_common::db_packets::db_settings;
#[cfg(feature = "statistics")]
use smol_db_common::statistics::DBStatistics;

/// Easy usable module containing everything needed to use the client library normally
pub mod prelude {
    pub use crate::client_error;
    pub use crate::client_error::ClientError::DBResponseError;
    pub use crate::SmolDbClient;
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

/// `SmolDbClient` struct used for communicating to the database.
/// This struct has implementations that allow for end to end communication with the database server.
pub struct SmolDbClient {
    socket: TcpStream,
}

impl SmolDbClient {
    /// Creates a new `SmolDBClient` struct connected to the ip address given.
    /// ```
    /// use smol_db_client::SmolDbClient;
    ///
    /// // create the new client
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    /// // client should be functional provided a database server was able to be connected to at the given location
    /// ```
    pub fn new(ip: &str) -> Result<Self, ClientError> {
        let socket = TcpStream::connect(ip);
        match socket {
            Ok(s) => Ok(Self { socket: s }),
            Err(err) => Err(UnableToConnect(err)),
        }
    }

    /// Reconnects the client, this will reset the session, which can be used to remove any key that was used.
    /// Or to reconnect in the event of a loss of connection
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// // disconnecting is optional between reconnects
    /// client.disconnect().unwrap();
    /// client.reconnect().unwrap();
    ///
    /// // as shown here
    ///
    /// client.reconnect().unwrap();
    ///
    /// ```
    pub fn reconnect(&mut self) -> Result<(), ClientError> {
        let ip = self.socket.peer_addr().map_err(UnableToConnect)?;
        let new_socket = TcpStream::connect(ip).map_err(UnableToConnect)?;
        self.socket = new_socket;
        Ok(())
    }

    /// Returns a result containing the peer address of this client
    pub fn get_connected_ip(&self) -> std::io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    /// Disconnects the socket from the database.
    /// ```
    /// use smol_db_client::SmolDbClient;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// // disconnect the client
    /// let _ = client.disconnect().expect("Failed to disconnect socket");
    /// ```
    pub fn disconnect(&self) -> std::io::Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    /// Deletes the data at the given db location, requires permissions to do so.
    /// ```
    /// use smol_db_client::client_error::ClientError;
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_packet_response::DBPacketResponseError;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_delete_data",DBSettings::default()).unwrap();
    /// let _ = client.write_db("doctest_delete_data","cool_data_location","cool_data");
    /// let read_data1 = client.read_db("doctest_delete_data","cool_data_location").unwrap().as_option().unwrap().to_string();
    /// assert_eq!(read_data1.as_str(),"cool_data");
    ///
    /// // delete the data at the given location
    /// let _ = client.delete_data("doctest_delete_data","cool_data_location").unwrap();
    /// let read_data2 = client.read_db("doctest_delete_data","cool_data_location");
    /// assert_eq!(read_data2.unwrap_err(),ClientError::DBResponseError(DBPacketResponseError::ValueNotFound)); // is err here means DBResponseError(ValueNotFound)
    ///
    /// let _ = client.delete_db("doctest_delete_data").unwrap();
    /// ```
    pub fn delete_data(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_data(db_name, db_location);
        self.send_packet(&packet)
    }

    /// Returns the `DBStatistics` struct if permissions allow it on a given db
    #[cfg(feature = "statistics")]
    pub fn get_stats(&mut self, db_name: &str) -> Result<DBStatistics, ClientError> {
        let packet = DBPacket::new_get_stats(db_name);
        let resp = self.send_packet(&packet)?;

        match resp {
            DBSuccessResponse::SuccessNoData => Err(BadPacket),
            DBSuccessResponse::SuccessReply(data) => {
                match serde_json::from_str::<DBStatistics>(&data) {
                    Ok(statistics) => Ok(statistics),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                }
            }
        }
    }

    /// Returns the role of the given client in the given db.
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db::Role;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_get_role",DBSettings::default()).unwrap();
    ///
    /// // get the given clients role from a db
    /// let role = client.get_role("doctest_get_role").unwrap();
    /// assert_eq!(role,Role::SuperAdmin);
    ///
    /// let _ = client.delete_db("doctest_get_role").unwrap();
    /// ```
    pub fn get_role(&mut self, db_name: &str) -> Result<Role, ClientError> {
        let packet = DBPacket::new_get_role(db_name);

        let resp = self.send_packet(&packet)?;

        match resp {
            DBSuccessResponse::SuccessNoData => Err(BadPacket),
            DBSuccessResponse::SuccessReply(data) => match serde_json::from_str::<Role>(&data) {
                Ok(role) => Ok(role),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Gets the `DBSettings` of the given DB.
    /// Error on IO error, or when database name does not exist, or when the user lacks permissions to view `DBSettings`.
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_get_db_settings",DBSettings::default()).unwrap();
    ///
    /// // get the db settings
    /// let settings = client.get_db_settings("doctest_get_db_settings").unwrap();
    /// assert_eq!(settings,DBSettings::default());
    ///
    /// let _ = client.delete_db("doctest_get_db_settings").unwrap();
    /// ```
    pub fn get_db_settings(&mut self, db_name: &str) -> Result<DBSettings, ClientError> {
        let packet = DBPacket::new_get_db_settings(db_name);

        let resp = self.send_packet(&packet)?;
        match resp {
            DBSuccessResponse::SuccessNoData => Err(BadPacket),
            DBSuccessResponse::SuccessReply(data) => {
                match serde_json::from_str::<DBSettings>(&data) {
                    Ok(db_settings) => Ok(db_settings),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                }
            }
        }
    }

    /// Sets the `DBSettings` of a given DB
    /// Error on IO Error, or when database does not exist, or when the user lacks permissions to set `DBSettings`
    /// ```
    /// use std::time::Duration;
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_set_db_settings",DBSettings::default()).unwrap();
    ///
    /// // set the new db settings
    /// let new_settings = DBSettings::new(Duration::from_secs(10),(true,false,true),(false,false,false),vec![],vec![]);
    /// let _ = client.set_db_settings("doctest_set_db_settings",new_settings.clone()).unwrap();
    ///
    /// let settings = client.get_db_settings("doctest_set_db_settings").unwrap();
    /// assert_eq!(settings,new_settings);
    ///
    /// let _ = client.delete_db("doctest_set_db_settings").unwrap();
    /// ```
    pub fn set_db_settings(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_db_settings(db_name, db_settings);
        self.send_packet(&packet)
    }

    /// Sets this clients access key within the DB Server. The server will persist the key until the session is disconnected, or connection is lost.
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// // sets the access key of the given client
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// ```
    pub fn set_access_key(
        &mut self,
        key: String,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_key(key);
        self.send_packet(&packet)
    }

    /// Sends a packet to the clients currently connected database and returns the result
    fn send_packet(
        &mut self,
        sent_packet: &DBPacket,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let ser_packet = sent_packet
            .serialize_packet()
            .map_err(|err| PacketSerializationError(Error::from(err)))?;
        self.socket
            .write(ser_packet.as_bytes())
            .map_err(SocketWriteError)?;
        let read_len = self.socket.read(&mut buf).map_err(SocketReadError)?;
        match serde_json::from_slice::<Result<DBSuccessResponse<String>, DBPacketResponseError>>(
            &buf[0..read_len],
        ) {
            Ok(thing) => thing.map_err(DBResponseError),
            Err(err) => Err(PacketDeserializationError(Error::from(err))),
        }
    }

    /// Creates a db through the client with the given name.
    /// Error on IO Error, or when the user lacks permissions to create a DB
    /// ```
    /// use std::time::Duration;
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_create_db",DBSettings::default()).unwrap();
    ///
    /// let _ = client.delete_db("doctest_create_db").unwrap();
    /// ```
    pub fn create_db(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_create_db(db_name, db_settings);
        let resp = self.send_packet(&packet)?;

        Ok(resp)
    }

    /// Writes to a db at the location specified, with the data given as a string.
    /// Returns the data in the location that was over written if there was any.
    /// Requires permissions to write to the given DB
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_write_data",DBSettings::default()).unwrap();
    ///
    /// // write the given data to the given location within the specified db
    /// let _ = client.write_db("doctest_write_data","cool_data_location","cool_data");
    ///
    /// let read_data1 = client.read_db("doctest_write_data","cool_data_location").unwrap().as_option().unwrap().to_string();
    /// assert_eq!(read_data1.as_str(),"cool_data");
    ///
    /// let _ = client.delete_db("doctest_write_data").unwrap();
    /// ```
    pub fn write_db(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_write(db_name, db_location, data);

        self.send_packet(&packet)
    }

    /// Reads from a db at the location specific.
    /// Returns an error if there is no data in the location.
    /// Requires permissions to read from the given DB
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_read_db",DBSettings::default()).unwrap();
    ///
    ///let _ = client.write_db("doctest_read_db","cool_data_location","cool_data");
    ///
    /// // read the given database at the given location
    /// let read_data1 = client.read_db("doctest_read_db","cool_data_location").unwrap().as_option().unwrap().to_string();
    /// assert_eq!(read_data1.as_str(),"cool_data");
    ///
    /// let _ = client.delete_db("doctest_read_db").unwrap();
    /// ```
    pub fn read_db(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_read(db_name, db_location);

        self.send_packet(&packet)
    }

    /// Deletes the given db by name.
    /// Requires super admin privileges on the given DB Server
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_delete_db",DBSettings::default()).unwrap();
    ///
    /// // delete the db with the given name
    /// let _ = client.delete_db("doctest_delete_db").unwrap();
    /// ```
    pub fn delete_db(&mut self, db_name: &str) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_db(db_name);

        self.send_packet(&packet)
    }

    /// Lists all the current databases available by name from the server
    /// Only error on IO Error
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_list_db1",DBSettings::default()).unwrap();
    ///
    /// // get list of databases currently on the server
    /// let list_of_dbs1 = client.list_db().unwrap();
    /// assert!(list_of_dbs1.contains(&DBPacketInfo::new("doctest_list_db1")));
    /// assert!(!list_of_dbs1.contains(&DBPacketInfo::new("doctest_list_db2")));
    ///
    /// let _ = client.create_db("doctest_list_db2",DBSettings::default()).unwrap();
    ///
    /// // newly created databases show up after getting another copy of the list
    /// let list_of_dbs2 = client.list_db().unwrap();
    /// assert!(list_of_dbs2.contains(&DBPacketInfo::new("doctest_list_db2")));
    /// assert!(list_of_dbs2.contains(&DBPacketInfo::new("doctest_list_db1")));
    ///
    /// let _ = client.delete_db("doctest_list_db1").unwrap();
    /// let _ = client.delete_db("doctest_list_db2").unwrap();
    /// ```
    pub fn list_db(&mut self) -> Result<Vec<DBPacketInfo>, ClientError> {
        let packet = DBPacket::new_list_db();

        let response = self.send_packet(&packet)?;

        match response {
            DBSuccessResponse::SuccessNoData => Err(BadPacket),
            DBSuccessResponse::SuccessReply(data) => {
                match serde_json::from_str::<Vec<DBPacketInfo>>(&data) {
                    Ok(thing) => Ok(thing),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                }
            }
        }
    }

    /// Get the hashmap of the contents of a database. Contents are always String:String for the hashmap.
    /// Requires list permissions on the given DB
    /// ```
    /// use smol_db_client::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_list_cont_db",DBSettings::default()).unwrap();
    ///
    ///let _ = client.write_db("doctest_list_cont_db","cool_data_location","cool_data");
    ///
    /// let contents = client.list_db_contents("doctest_list_cont_db").unwrap();
    /// assert_eq!(contents.len(),1);
    /// assert_eq!(contents.get("cool_data_location").unwrap().as_str(),"cool_data");
    ///
    /// let _ = client.delete_db("doctest_list_cont_db").unwrap();
    /// ```
    pub fn list_db_contents(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, String>, ClientError> {
        let packet = DBPacket::new_list_db_contents(db_name);

        let response = self.send_packet(&packet)?;

        match response {
            DBSuccessResponse::SuccessNoData => Err(BadPacket),
            DBSuccessResponse::SuccessReply(data) => {
                match serde_json::from_str::<HashMap<String, String>>(&data) {
                    Ok(thing) => Ok(thing),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                }
            }
        }
    }

    /// Lists the given db's contents, deserializing the contents into a hash map.
    pub fn list_db_contents_generic<T>(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        let contents = self.list_db_contents(db_name)?;
        let mut converted_contents: HashMap<String, T> = HashMap::new();
        for (key, value) in contents {
            match serde_json::from_str::<T>(&value) {
                Ok(thing) => {
                    converted_contents.insert(key, thing);
                }
                Err(err) => {
                    return Err(PacketDeserializationError(Error::from(err)));
                }
            }
        }
        Ok(converted_contents)
    }

    /// Writes to the db while serializing the given data, returning the data at the location given and deserialized to the same type.
    pub fn write_db_generic<T>(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: T,
    ) -> Result<DBSuccessResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match serde_json::to_string(&data) {
            Ok(ser_data) => match self.write_db(db_name, db_location, &ser_data) {
                Ok(response) => match response {
                    DBSuccessResponse::SuccessNoData => Ok(DBSuccessResponse::SuccessNoData),
                    DBSuccessResponse::SuccessReply(data_string) => {
                        match serde_json::from_str::<T>(&data_string) {
                            Ok(thing) => Ok(DBSuccessResponse::SuccessReply(thing)),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                },
                Err(err) => Err(err),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        }
    }

    /// Reads from db and tries to deserialize the content at the location to the given generic
    pub fn read_db_generic<T>(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match self.read_db(db_name, db_location) {
            Ok(data) => match data {
                DBSuccessResponse::SuccessNoData => Ok(DBSuccessResponse::SuccessNoData),
                DBSuccessResponse::SuccessReply(read_data) => {
                    match serde_json::from_str::<T>(&read_data) {
                        Ok(data) => Ok(DBSuccessResponse::SuccessReply(data)),
                        Err(err) => Err(PacketDeserializationError(Error::from(err))),
                    }
                }
            },
            Err(err) => Err(err),
        }
    }
}
