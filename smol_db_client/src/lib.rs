//! Library contain the structs that manage the client to connect to smol_db

use crate::client_error::ClientError;
use crate::ClientError::{
    BadPacket, PacketDeserializationError, PacketSerializationError, SocketReadError,
    SocketWriteError, UnableToConnect,
};
use serde::{Deserialize, Serialize};
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpStream};
use smol_db_common::db_packets::db_settings::DBSettings;

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

    /// Creates a db through the client with the given name.
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
                                            ClientError::DBResponseError(db_response_error.clone()),
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
                                    Err(ClientError::DBResponseError(err))
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
                                    Err(ClientError::DBResponseError(err))
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
                    DBPacketResponse::Error(err) => Err(ClientError::DBResponseError(err)),
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
                DBPacketResponse::Error(err) => Err(ClientError::DBResponseError(err)),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::{Client, ClientError};
    use serde::{Deserialize, Serialize};
    use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
    use std::thread;
    use std::time::Duration;
    use smol_db_common::db_packets::db_settings::DBSettings;

    #[test]
    fn test_client() {
        let mut client = Client::new("localhost:8222").unwrap();

        let create_response = client.create_db("test2", DBSettings::default()).unwrap();

        match create_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

        let data = "this_is_data";
        let write_response = client.write_db("test2", "location1", data).unwrap();

        match write_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Write db failed.")
            }
        }

        let read_response = client.read_db("test2", "location1").unwrap();

        match read_response {
            DBPacketResponse::SuccessReply(response_data) => {
                assert_eq!(&response_data, data);
            }
            _ => {
                panic!("data response was not as expected");
            }
        }

        let data2 = "this_is_not_data";
        let write_response2 = client.write_db("test2", "location1", data2).unwrap();

        match write_response2 {
            DBPacketResponse::SuccessReply(previous_data) => {
                assert_eq!(data, &previous_data)
            }
            _ => {
                panic!("Write db 2 failed.")
            }
        }

        let read_response2 = client.read_db("test2", "location1").unwrap();

        match read_response2 {
            DBPacketResponse::SuccessReply(response_data) => {
                assert_eq!(&response_data, data2);
            }
            _ => {
                panic!("data response was not as expected");
            }
        }

        let delete_response = client.delete_db("test2").unwrap();

        match delete_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Delete db failed.")
            }
        }
    }

    #[derive(PartialEq, Eq, Deserialize, Serialize, Clone, Debug)]
    struct TestStruct {
        a: u32,
        b: bool,
        c: i32,
        d: String,
    }

    #[test]
    fn test_generics_client() {
        let mut client = Client::new("localhost:8222").unwrap();
        let test_data1 = TestStruct {
            a: 10,
            b: false,
            c: -500,
            d: "test_data123".to_string(),
        };

        let test_data2 = TestStruct {
            a: 15,
            b: true,
            c: 495,
            d: "123_test_data".to_string(),
        };

        let create_db_response = client
            .create_db("test_generics", DBSettings::default())
            .unwrap();

        match create_db_response {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let write_db_response1 = client
            .write_db_generic("test_generics", "location1", test_data1.clone())
            .unwrap();

        match write_db_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err)
            }
            _ => {}
        }

        let read_db_response1 = client
            .read_db_generic::<TestStruct>("test_generics", "location1")
            .unwrap();

        match read_db_response1 {
            DBPacketResponse::SuccessReply(received_struct) => {
                assert_eq!(received_struct, test_data1);
            }
            _ => {
                panic!("Read db error 1")
            }
        }

        let write_db_response2 = client
            .write_db_generic::<TestStruct>("test_generics", "location1", test_data2.clone())
            .unwrap();

        match write_db_response2 {
            DBPacketResponse::SuccessReply(previous_struct) => {
                assert_eq!(previous_struct, test_data1);
            }
            _ => {
                panic!("Write db error 2")
            }
        }

        let read_db_response2 = client
            .read_db_generic::<TestStruct>("test_generics", "location1")
            .unwrap();

        match read_db_response2 {
            DBPacketResponse::SuccessReply(received_struct) => {
                assert_eq!(received_struct, test_data2);
            }
            _ => {
                panic!("Read db error 1")
            }
        }

        let delete_db_response = client.delete_db("test_generics").unwrap();

        match delete_db_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Unable to delete db");
            }
        }
    }

    #[test]
    fn test_list_db() {
        let mut client = Client::new("localhost:8222").unwrap();

        let create_db_response1 = client
            .create_db("test_db_1", DBSettings::default())
            .unwrap();

        match create_db_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let create_db_response2 = client
            .create_db("test_db_2", DBSettings::default())
            .unwrap();

        match create_db_response2 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let list_db_response = client.list_db().unwrap();

        assert!(list_db_response.clone().len() >= 2);

        assert!(list_db_response.contains(&DBPacketInfo::new("test_db_1")));
        assert!(list_db_response.contains(&DBPacketInfo::new("test_db_2")));

        let delete_db_response1 = client.delete_db("test_db_1").unwrap();

        match delete_db_response1 {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Unable to delete db 1");
            }
        }

        let delete_db_response2 = client.delete_db("test_db_2").unwrap();

        match delete_db_response2 {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Unable to delete db 2");
            }
        }
    }

    #[test]
    fn test_empty_db_list() {
        let mut client = Client::new("localhost:8222").unwrap();
        loop {
            // continue looping indefinitely until we manage to read an empty list db, verifying that serialization works even when the list would be empty.
            let list = client.list_db().unwrap();
            let len = list.len();
            thread::sleep(Duration::from_millis(1)); // wait a small amount of time between lists so we dont dominate the thread pool on the server.
            if len == 0 {
                // if we find a 0 length return, then we have clearly not panicked and can stop looping, allowing the test to be successful
                break;
            }
        }
    }

    #[test]
    fn test_list_db_contents() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_name = "test_db_contents1";

        let create_db_response1 = client.create_db(db_name, DBSettings::default()).unwrap();

        match create_db_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let write_response1 = client.write_db(db_name, "location1", "123").unwrap();
        match write_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let write_response2 = client.write_db(db_name, "location2", "456").unwrap();
        match write_response2 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let list_db_contents_response = client.list_db_contents(db_name).unwrap();

        assert_eq!(list_db_contents_response.get("location1").unwrap(), "123");
        assert_eq!(list_db_contents_response.get("location2").unwrap(), "456");

        let delete_db_response = client.delete_db(db_name).unwrap();

        match delete_db_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Unable to delete db");
            }
        }
    }

    #[test]
    fn test_list_db_contents_empty() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_name = "test_db_contents_empty1";

        let create_db_response1 = client.create_db(db_name, DBSettings::default()).unwrap();

        match create_db_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let contents = client.list_db_contents(db_name).unwrap();

        assert_eq!(contents.is_empty(), true); // contents should be empty as no write operations occurred.

        let delete_db_response = client.delete_db(db_name).unwrap();

        match delete_db_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Unable to delete db");
            }
        }
    }

    #[test]
    fn test_list_db_contents_generic() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_name = "test_list_db_contents_generic1";
        let test_data1 = TestStruct {
            a: 10,
            b: false,
            c: -500,
            d: "test_data123".to_string(),
        };

        let test_data2 = TestStruct {
            a: 15,
            b: true,
            c: 495,
            d: "123_test_data".to_string(),
        };

        let create_response = client.create_db(db_name, DBSettings::default()).unwrap();
        match create_response {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let write_response1 = client
            .write_db_generic(db_name, "location1", test_data1.clone())
            .unwrap();
        match write_response1 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let write_response2 = client
            .write_db_generic(db_name, "location2", test_data2.clone())
            .unwrap();
        match write_response2 {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }

        let list = client
            .list_db_contents_generic::<TestStruct>(db_name)
            .unwrap();

        assert_eq!(list.len(), 2);

        assert_eq!(list.get("location1").unwrap().clone(), test_data1);
        assert_eq!(list.get("location2").unwrap().clone(), test_data2);

        let delete_response = client.delete_db(db_name).unwrap();
        match delete_response {
            DBPacketResponse::Error(err) => {
                panic!("{:?}", err);
            }
            _ => {}
        }
    }
}
