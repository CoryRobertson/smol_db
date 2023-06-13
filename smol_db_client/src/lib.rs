//! Library contain the structs that manage the client to connect to smol_db

use crate::client_error::ClientError;
use crate::client_error::ClientError::DBResponseError;
use crate::ClientError::{
    BadPacket, PacketDeserializationError, PacketSerializationError, SocketReadError,
    SocketWriteError, UnableToConnect,
};
use serde::{Deserialize, Serialize};
use smol_db_common::db::Role;
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use smol_db_common::db_packets::db_settings::DBSettings;
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpStream};

pub mod client_error;

/// Client struct used for communicating to the database.
pub struct Client {
    socket: TcpStream,
}

impl Client {
    /// Creates a new SmolDBClient struct connected to the ip address given.
    pub fn new(ip: &str) -> Result<Self, ClientError> {
        let socket = TcpStream::connect(ip);
        match socket {
            Ok(s) => Ok(Self { socket: s }),
            Err(err) => Err(UnableToConnect(err)),
        }
    }

    /// Disconnects the socket from the database.
    pub fn disconnect(self) -> std::io::Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    pub fn get_role(&mut self, db_name: &str) -> Result<Role, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_get_role(db_name);
        match packet1.serialize_packet() {
            Ok(packet_ser) => match self.socket.write(packet_ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_length) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(
                            &buf[0..read_length],
                        ) {
                            Ok(response) => match response {
                                DBPacketResponse::SuccessNoData => Err(BadPacket),
                                DBPacketResponse::SuccessReply(data) => {
                                    match serde_json::from_str::<Role>(&data) {
                                        Ok(role) => Ok(role),
                                        Err(err) => {
                                            Err(PacketDeserializationError(Error::from(err)))
                                        }
                                    }
                                }
                                DBPacketResponse::Error(err) => Err(DBResponseError(err)),
                            },
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        }
    }

    /// Gets the DBSettings of the given DB.
    /// Error on IO error, or when database name does not exist, or when the user lacks permissions to view DBSettings.
    pub fn get_db_settings(&mut self, db_name: &str) -> Result<DBSettings, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_get_db_settings(db_name);
        return match packet1.serialize_packet() {
            Ok(packet_ser) => match self.socket.write(packet_ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_size) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_size])
                        {
                            Ok(resp) => match resp {
                                DBPacketResponse::SuccessNoData => Err(BadPacket),
                                DBPacketResponse::SuccessReply(data) => {
                                    match serde_json::from_str::<DBSettings>(&data) {
                                        Ok(db_settings) => Ok(db_settings),
                                        Err(err) => {
                                            Err(PacketDeserializationError(Error::from(err)))
                                        }
                                    }
                                }
                                DBPacketResponse::Error(err) => Err(DBResponseError(err)),
                            },
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Sets the DBSettings of a given DB
    /// Error on IO Error, or when database does not exist, or when the user lacks permissions to set DBSettings
    pub fn set_db_settings(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_set_db_settings(db_name, db_settings);
        return match packet1.serialize_packet() {
            Ok(packet_ser) => match self.socket.write(packet_ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_size) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_size])
                        {
                            Ok(resp) => Ok(resp),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Sets this clients access key within the DB Server. The server will persist the key until the session is disconnected, or connection is lost.
    pub fn set_access_key(&mut self, key: String) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_set_key(key);
        return match packet1.serialize_packet() {
            Ok(packet_ser) => match self.socket.write(packet_ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_len) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_len])
                        {
                            Ok(packet_response) => Ok(packet_response),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Creates a db through the client with the given name.
    /// Error on IO Error, or when the user lacks permissions to create a DB
    pub fn create_db(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_create_db(db_name, db_settings);
        return match packet1.serialize_packet() {
            Ok(pack_bytes) => {
                let write_result = self.socket.write(pack_bytes.as_bytes());
                match write_result {
                    Ok(_) => {
                        let read_result = self.socket.read(&mut buf);
                        match read_result {
                            Ok(read_size) => {
                                match serde_json::from_slice::<DBPacketResponse<String>>(
                                    &buf[0..read_size],
                                ) {
                                    Ok(response) => match &response {
                                        DBPacketResponse::SuccessNoData => Ok(response),
                                        DBPacketResponse::SuccessReply(_) => Ok(response),
                                        DBPacketResponse::Error(db_response_error) => Err(
                                            DBResponseError(db_response_error.clone()),
                                        ),
                                    },
                                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                                }
                            }
                            Err(err) => Err(SocketReadError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketWriteError(Error::from(err))),
                }
            }
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Writes to a db at the location specified, with the data given as a string.
    /// Returns the data in the location that was over written if there was any.
    /// Requires permissions to write to the given DB
    pub fn write_db(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: &str,
    ) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet = DBPacket::new_write(db_name, db_location, data);
        return match packet.serialize_packet() {
            Ok(ser) => match self.socket.write(ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_length) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(
                            &buf[0..read_length],
                        ) {
                            Ok(response) => Ok(response),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Reads from a db at the location specific.
    /// Returns an error if there is no data in the location.
    /// Requires permissions to read from the given DB
    pub fn read_db(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet = DBPacket::new_read(db_name, db_location);
        return match packet.serialize_packet() {
            Ok(ser) => match self.socket.write(ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_length) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(
                            &buf[0..read_length],
                        ) {
                            Ok(response) => Ok(response),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Deletes the given db by name.
    /// Requires super admin privileges on the given DB Server
    pub fn delete_db(&mut self, db_name: &str) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet = DBPacket::new_delete_db(db_name);
        return match packet.serialize_packet() {
            Ok(ser) => match self.socket.write(ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_length) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(
                            &buf[0..read_length],
                        ) {
                            Ok(response) => Ok(response),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Lists all the current databases available by name from the server
    /// Only error on IO Error
    pub fn list_db(&mut self) -> Result<Vec<DBPacketInfo>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet = DBPacket::new_list_db();

        return match packet.serialize_packet() {
            Ok(ser) => match self.socket.write(ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_len) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_len])
                        {
                            Ok(response) => match response {
                                DBPacketResponse::SuccessNoData => Err(BadPacket),
                                DBPacketResponse::SuccessReply(data) => {
                                    match serde_json::from_str::<Vec<DBPacketInfo>>(&data) {
                                        Ok(thing) => Ok(thing),
                                        Err(err) => {
                                            Err(PacketDeserializationError(Error::from(err)))
                                        }
                                    }
                                }
                                DBPacketResponse::Error(err) => {
                                    Err(DBResponseError(err))
                                }
                            },
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

    /// Get the hashmap of the contents of a database. Contents are always String:String for the hashmap.
    /// Requires list permissions on the given DB
    pub fn list_db_contents(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet = DBPacket::new_list_db_contents(db_name);

        return match packet.serialize_packet() {
            Ok(ser) => match self.socket.write(ser.as_bytes()) {
                Ok(_) => match self.socket.read(&mut buf) {
                    Ok(read_len) => {
                        match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_len])
                        {
                            Ok(resp) => match resp {
                                DBPacketResponse::SuccessNoData => Err(BadPacket),
                                DBPacketResponse::SuccessReply(data) => {
                                    match serde_json::from_str::<HashMap<String, String>>(&data) {
                                        Ok(thing) => Ok(thing),
                                        Err(err) => {
                                            Err(PacketDeserializationError(Error::from(err)))
                                        }
                                    }
                                }
                                DBPacketResponse::Error(err) => {
                                    Err(DBResponseError(err))
                                }
                            },
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    Err(err) => Err(SocketReadError(err)),
                },
                Err(err) => Err(SocketWriteError(err)),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        };
    }

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
    ) -> Result<DBPacketResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match serde_json::to_string(&data) {
            Ok(ser_data) => match self.write_db(db_name, db_location, &ser_data) {
                Ok(response) => match response {
                    DBPacketResponse::SuccessNoData => Ok(DBPacketResponse::SuccessNoData),
                    DBPacketResponse::SuccessReply(data_string) => {
                        match serde_json::from_str::<T>(&data_string) {
                            Ok(thing) => Ok(DBPacketResponse::SuccessReply(thing)),
                            Err(err) => Err(PacketDeserializationError(Error::from(err))),
                        }
                    }
                    DBPacketResponse::Error(err) => Err(DBResponseError(err)),
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
    ) -> Result<DBPacketResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match self.read_db(db_name, db_location) {
            Ok(data) => match data {
                DBPacketResponse::SuccessNoData => Ok(DBPacketResponse::SuccessNoData),
                DBPacketResponse::SuccessReply(read_data) => {
                    match serde_json::from_str::<T>(&read_data) {
                        Ok(data) => Ok(DBPacketResponse::SuccessReply(data)),
                        Err(err) => Err(PacketDeserializationError(Error::from(err))),
                    }
                }
                DBPacketResponse::Error(err) => Err(DBResponseError(err)),
            },
            Err(err) => Err(err),
        }
    }
}
