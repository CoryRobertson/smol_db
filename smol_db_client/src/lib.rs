//! Library contain the structs that manage the client to connect to smol_db

use crate::ClientError::{
    PacketDeserializationError, PacketSerializationError, SocketWriteError, UnableToConnect,
};
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

//TODO: write a smol_db_client struct that facilitates all actions, as abstract as possible. It should be created using a factory function that takes in the desired ip address.
//  The struct should contain a tcp socket, the previously input ip address. These should all be non-public, and everything relating to these objects should be fully wrapped.
//  It should maintain the connection, and allow for abstract functions like:
//  create_db()
//  delete_db()
//  write_db()
//  read_db()

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
    pub fn create_db(
        &mut self,
        db_name: &str,
        invalidation_time: Duration,
    ) -> Result<DBPacketResponse<String>, ClientError> {
        // TODO: untested function
        let mut buf: [u8; 1024] = [0; 1024];
        let packet1 = DBPacket::new_create_db(db_name, invalidation_time);

        return match packet1.serialize_packet() {
            Ok(pack_bytes) => {
                let write_result = self.socket.write(&pack_bytes);
                match write_result {
                    Ok(_) => {
                        let read_result = self.socket.read(&mut buf);
                        match read_result {
                            Ok(read_size) => {
                                match DBPacketResponse::deserialize_packet(&buf[0..read_size]) {
                                    Ok(response) => {
                                        match &response {

                                            DBPacketResponse::SuccessNoData => { Ok(response)}

                                            DBPacketResponse::SuccessReply(_) => { Ok(response)}

                                            DBPacketResponse::Error(db_response_error) => {
                                                Err(ClientError::DBResponseError(db_response_error.clone()))
                                            }
                                        }
                                    },
                                    Err(err) => Err(PacketDeserializationError(err)),
                                }
                            }
                            Err(err) => Err(SocketWriteError(err)),
                        }
                    }
                    Err(err) => Err(SocketWriteError(err)),
                }
            }
            Err(err) => Err(PacketSerializationError(err)),
        };
    }
}


pub enum ClientError {
    UnableToConnect(Error),
    PacketSerializationError(rmp_serde::encode::Error),
    SocketWriteError(Error),
    PacketDeserializationError(rmp_serde::decode::Error),
    DBResponseError(DBPacketResponseError)
}

#[cfg(test)]
mod tests {

}