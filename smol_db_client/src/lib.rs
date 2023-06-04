//! Library contain the structs that manage the client to connect to smol_db

use crate::ClientError::{PacketDeserializationError, PacketSerializationError, SocketReadError, SocketWriteError, UnableToConnect};
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

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

    /// Creates a db through the client with the given name.
    pub fn create_db(&mut self, db_name: &str, invalidation_time: Duration, ) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_create_db(db_name, invalidation_time);
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
                                    Ok(response) => {
                                        match &response {
                                            DBPacketResponse::SuccessNoData => { Ok(response)}
                                            DBPacketResponse::SuccessReply(_) => { Ok(response)}
                                            DBPacketResponse::Error(db_response_error) => {
                                                Err(ClientError::DBResponseError(db_response_error.clone()))
                                            }
                                        }
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
    pub fn write_db(&mut self, db_name: &str, db_location: &str, data: &str) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8 ; 1024] = [0; 1024];
        let packet = DBPacket::new_write(db_name,db_location,data);
        return match packet.serialize_packet() {
            Ok(ser) => {
                match self.socket.write(ser.as_bytes()) {
                    Ok(_) => {
                        match self.socket.read(&mut buf) {
                            Ok(read_length) => {
                                match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_length]) {
                                    Ok(response) => {
                                        Ok(response)
                                    }
                                    Err(err) => {
                                        Err(PacketDeserializationError(Error::from(err)))
                                    }
                                }
                            }
                            Err(err) => {
                                Err(SocketReadError(err))
                            }
                        }
                    }
                    Err(err) => {
                        Err(SocketWriteError(err))
                    }
                }
            }
            Err(err) => {
                Err(PacketSerializationError(Error::from(err)))
            }
        }
    }

    /// Reads from a db at the location specific.
    /// Returns an error if there is no data in the location.
    pub fn read_db(&mut self, db_name: &str, db_location: &str) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8 ; 1024] = [0; 1024];
        let packet = DBPacket::new_read(db_name,db_location);
        return match packet.serialize_packet() {
            Ok(ser) => {
                match self.socket.write(ser.as_bytes()) {
                    Ok(_) => {
                        match self.socket.read(&mut buf) {
                            Ok(read_length) => {
                                match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_length]) {
                                    Ok(response) => {
                                        Ok(response)
                                    }
                                    Err(err) => {
                                        Err(PacketDeserializationError(Error::from(err)))
                                    }
                                }
                            }
                            Err(err) => {
                                Err(SocketReadError(err))
                            }
                        }
                    }
                    Err(err) => {
                        Err(SocketWriteError(err))
                    }
                }
            }
            Err(err) => {
                Err(PacketSerializationError(Error::from(err)))
            }
        }
    }

    /// Deletes the given db by name.
    pub fn delete_db(&mut self, db_name: &str) -> Result<DBPacketResponse<String>, ClientError> {
        let mut buf: [u8 ; 1024] = [0; 1024];
        let packet = DBPacket::new_delete_db(db_name);
        return match packet.serialize_packet() {
            Ok(ser) => {
                match self.socket.write(ser.as_bytes()) {
                    Ok(_) => {
                        match self.socket.read(&mut buf) {
                            Ok(read_length) => {
                                match serde_json::from_slice::<DBPacketResponse<String>>(&buf[0..read_length]) {
                                    Ok(response) => {
                                        Ok(response)
                                    }
                                    Err(err) => {
                                        Err(PacketDeserializationError(Error::from(err)))
                                    }
                                }
                            }
                            Err(err) => {
                                Err(SocketReadError(err))
                            }
                        }
                    }
                    Err(err) => {
                        Err(SocketWriteError(err))
                    }
                }
            }
            Err(err) => {
                Err(PacketSerializationError(Error::from(err)))
            }
        }
    }

    // TODO: use generic serialization to write anything to db, and read anything from db?
}

#[derive(Debug)]
pub enum ClientError {
    UnableToConnect(Error),
    PacketSerializationError(Error),
    SocketWriteError(Error),
    SocketReadError(Error),
    PacketDeserializationError(Error),
    DBResponseError(DBPacketResponseError)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use std::time::Duration;
    use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
    use crate::{Client, ClientError};

    #[test]
    fn test_client() {
        let mut client = Client::new("localhost:8222").unwrap();

        let create_response = client.create_db("test2",Duration::from_secs(30)).unwrap();

        match create_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

        let data = "this_is_data";
        let write_response = client.write_db("test2","location1",data).unwrap();

        match write_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Write db failed.")
            }
        }

        let read_response = client.read_db("test2", "location1").unwrap();

        match read_response {
            DBPacketResponse::SuccessReply(response_data) => {
                assert_eq!(&response_data,data);
            }
            _ => {
                panic!("data response was not as expected");
            }
        }

        let data2 = "this_is_not_data";
        let write_response2 = client.write_db("test2","location1",data2).unwrap();

        match write_response2 {
            DBPacketResponse::SuccessReply(previous_data) => {
                assert_eq!(data,&previous_data)
            }
            _ => {
                panic!("Write db 2 failed.")
            }
        }

        let read_response2 = client.read_db("test2", "location1").unwrap();

        match read_response2 {
            DBPacketResponse::SuccessReply(response_data) => {
                assert_eq!(&response_data,data2);
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
}
