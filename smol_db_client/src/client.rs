use crate::client_error::ClientError;
use crate::client_error::ClientError::{
    BadPacket, EncryptionSetupError, KeyGenerationError, PacketDeserializationError,
    PacketEncryptionError, PacketSerializationError, SocketReadError, SocketWriteError,
    UnableToConnect,
};
#[cfg(not(feature = "async"))]
use crate::prelude::TableIter;
use crate::prelude::{DBResponseError};
use serde::{Deserialize, Serialize};
use smol_db_common::db::Role;
use smol_db_common::encryption::client_encrypt::ClientKey;
use smol_db_common::prelude::{
    DBPacket, DBPacketInfo, DBPacketResponseError, DBSettings, DBSuccessResponse, RsaPublicKey,
    SuccessNoData, SuccessReply,
};
#[cfg(feature = "statistics")]
use smol_db_common::statistics::DBStatistics;
use std::collections::HashMap;
use std::io::Error;
#[cfg(not(feature = "async"))]
use std::io::{Read, Write};
#[cfg(not(feature = "async"))]
use std::net::Shutdown;

use std::net::SocketAddr;

#[cfg(feature = "async")]
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};
use tracing::{error, info, warn};
#[cfg(not(feature = "async"))]
use tracing::debug;

#[cfg(not(feature = "async"))]
use std::net::TcpStream;

#[derive(Debug)]
/// `SmolDbClient` struct used for communicating to the database.
/// This struct has implementations that allow for end to end communication with the database server.
pub struct SmolDbClient {
    socket: TcpStream,
    encryption: Option<ClientKey>,
}

impl SmolDbClient {

    #[allow(dead_code)]
    pub(crate) fn get_socket(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    #[cfg(not(feature = "async"))]
    pub fn stream_table(&mut self, table_name: &str) -> Result<TableIter, ClientError> {
        let packet = DBPacket::new_stream_table(table_name);

        debug!("Sending packet");

        let resp = self.send_packet(&packet)?;

        debug!("Sent packet: {}", resp);
        let table_iter = TableIter(self);

        Ok(table_iter)
    }

    /// Creates a new `SmolDBClient` struct connected to the ip address given.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
    ///
    /// // create the new client
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    /// // client should be functional provided a database server was able to be connected to at the given location
    /// ```
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn new(ip: &str) -> Result<Self, ClientError> {
        info!("Creating new client");
        let socket = TcpStream::connect(ip);
        match socket {
            Ok(s) => Ok(Self {
                socket: s,
                encryption: None,
            }),
            Err(err) => {
                error!("Error creating client: {}", err);
                Err(UnableToConnect(err))
            }
        }
    }

    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn new(ip: &str) -> Result<Self, ClientError> {
        info!("Creating new client");
        let socket = TcpStream::connect(ip).await;
        match socket {
            Ok(s) => Ok(Self {
                socket: s,
                encryption: None,
            }),
            Err(err) => {
                error!("Error creating client: {}", err);
                Err(UnableToConnect(err))
            }
        }
    }

    /// Requests the server to use encryption for communication. Encryption is done both ways, and is done using RSA with a 2048-bit key
    /// This function is slow due to large rsa key size ~1-4 seconds to generate the key
    /// Encryption is done invisibly.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
    /// use smol_db_common::prelude::DBSettings;
    ///
    /// let key = "test_key_123";
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    /// client.set_access_key(key.to_string()).unwrap();
    /// client.setup_encryption().unwrap(); // encryption is done invisibly after it is setup, nothing else is needed :)
    /// client.create_db("docsetup_encryption_test",DBSettings::default()).unwrap();
    /// let _ = client.delete_db("docsetup_encryption_test").unwrap();
    /// ```
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn setup_encryption(&mut self) -> Result<DBSuccessResponse<String>, ClientError> {
        info!("Setting up encryption on client");
        let server_pub_key_ser = self
            .send_packet(&DBPacket::SetupEncryption)?
            .as_option()
            .ok_or(EncryptionSetupError)?
            .to_string();
        let server_pub_key = serde_json::from_str::<RsaPublicKey>(&server_pub_key_ser)
            .map_err(|err| PacketDeserializationError(Error::from(err)))?;
        // this function is really slow due to long key length generation, this can be modified if needed, but at the moment, this is ok.
        let pri_key = ClientKey::new(server_pub_key).map_err(KeyGenerationError)?;
        let pub_client_key = pri_key.get_pub_key().clone();
        self.encryption = Some(pri_key);
        let resp = self.send_packet(&DBPacket::PubKey(pub_client_key));
        if resp.is_err() {
            self.encryption = None;
            error!("Response from server: {:?}", resp);
        } else {
            info!("Response from server: {:?}", resp);
        }
        resp
    }

    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn setup_encryption(&mut self) -> Result<DBSuccessResponse<String>, ClientError> {
        info!("Setting up encryption on client");
        let server_pub_key_ser = self
            .send_packet(&DBPacket::SetupEncryption)
            .await?
            .as_option()
            .ok_or(EncryptionSetupError)?
            .to_string();
        let server_pub_key = serde_json::from_str::<RsaPublicKey>(&server_pub_key_ser)
            .map_err(|err| PacketDeserializationError(Error::from(err)))?;
        // this function is really slow due to long key length generation, this can be modified if needed, but at the moment, this is ok.
        let pri_key = ClientKey::new(server_pub_key).map_err(KeyGenerationError)?;
        let pub_client_key = pri_key.get_pub_key().clone();
        self.encryption = Some(pri_key);
        let resp = self.send_packet(&DBPacket::PubKey(pub_client_key)).await;
        if resp.is_err() {
            error!("Response from server: {:?}", resp);
            self.encryption = None;
        } else {
            info!("Response from server: {:?}", resp);
        }
        resp
    }

    /// Returns true if end-to-end encryption is enabled
    #[tracing::instrument]
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption.is_some()
    }

    /// Reconnects the client, this will reset the session, which can be used to remove any key that was used.
    /// Or to reconnect in the event of a loss of connection
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn reconnect(&mut self) -> Result<(), ClientError> {
        info!("Reconnecting client to database");
        let ip = self.socket.peer_addr().map_err(UnableToConnect)?;
        let new_socket = TcpStream::connect(ip).map_err(UnableToConnect)?;
        self.socket = new_socket;
        Ok(())
    }

    /// Reconnects the client, this will reset the session, which can be used to remove any key that was used.
    /// Or to reconnect in the event of a loss of connection
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn reconnect(&mut self) -> Result<(), ClientError> {
        info!("Reconnecting client to database");
        let ip = self.socket.peer_addr().map_err(UnableToConnect)?;
        let new_socket = TcpStream::connect(ip).await.map_err(UnableToConnect)?;
        self.socket = new_socket;
        Ok(())
    }

    /// Returns a result containing the peer address of this client
    #[tracing::instrument]
    pub fn get_connected_ip(&self) -> std::io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    /// Disconnects the socket from the database.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// // disconnect the client
    /// let _ = client.disconnect().expect("Failed to disconnect socket");
    /// ```
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn disconnect(&self) -> std::io::Result<()> {
        info!("Disconnecting client from database");
        self.socket.shutdown(Shutdown::Both)
    }

    /// Disconnects the socket from the database.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn disconnect(&mut self) -> std::io::Result<()> {
        info!("Disconnecting client from database");
        self.socket.shutdown().await
    }

    /// Deletes the data at the given db location, requires permissions to do so.
    /// ```
    /// use smol_db_client::client_error::ClientError;
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn delete_data(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_data(db_name, db_location);
        self.send_packet(&packet)
    }

    /// Deletes the data at the given db location, requires permissions to do so.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn delete_data(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_data(db_name, db_location);
        self.send_packet(&packet).await
    }

    /// Returns the `DBStatistics` struct if permissions allow it on a given db
    #[cfg(feature = "statistics")]
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn get_stats(&mut self, db_name: &str) -> Result<DBStatistics, ClientError> {
        let packet = DBPacket::new_get_stats(db_name);
        let resp = self.send_packet(&packet)?;

        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<DBStatistics>(&data) {
                Ok(statistics) => Ok(statistics),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Returns the `DBStatistics` struct if permissions allow it on a given db
    #[cfg(feature = "statistics")]
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn get_stats(&mut self, db_name: &str) -> Result<DBStatistics, ClientError> {
        let packet = DBPacket::new_get_stats(db_name);
        let resp = self.send_packet(&packet).await?;

        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<DBStatistics>(&data) {
                Ok(statistics) => Ok(statistics),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Returns the role of the given client in the given db.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn get_role(&mut self, db_name: &str) -> Result<Role, ClientError> {
        let packet = DBPacket::new_get_role(db_name);

        let resp = self.send_packet(&packet)?;

        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<Role>(&data) {
                Ok(role) => Ok(role),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Returns the role of the given client in the given db.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn get_role(&mut self, db_name: &str) -> Result<Role, ClientError> {
        let packet = DBPacket::new_get_role(db_name);

        let resp = self.send_packet(&packet).await?;

        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<Role>(&data) {
                Ok(role) => Ok(role),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Gets the `DBSettings` of the given DB.
    /// Error on IO error, or when database name does not exist, or when the user lacks permissions to view `DBSettings`.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn get_db_settings(&mut self, db_name: &str) -> Result<DBSettings, ClientError> {
        let packet = DBPacket::new_get_db_settings(db_name);

        let resp = self.send_packet(&packet)?;
        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<DBSettings>(&data) {
                Ok(db_settings) => Ok(db_settings),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Gets the `DBSettings` of the given DB.
    /// Error on IO error, or when database name does not exist, or when the user lacks permissions to view `DBSettings`.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn get_db_settings(&mut self, db_name: &str) -> Result<DBSettings, ClientError> {
        let packet = DBPacket::new_get_db_settings(db_name);

        let resp = self.send_packet(&packet).await?;
        match resp {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<DBSettings>(&data) {
                Ok(db_settings) => Ok(db_settings),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Sets the `DBSettings` of a given DB
    /// Error on IO Error, or when database does not exist, or when the user lacks permissions to set `DBSettings`
    /// ```
    /// use std::time::Duration;
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn set_db_settings(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_db_settings(db_name, db_settings);
        self.send_packet(&packet)
    }

    /// Sets the `DBSettings` of a given DB
    /// Error on IO Error, or when database does not exist, or when the user lacks permissions to set `DBSettings`
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn set_db_settings(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_db_settings(db_name, db_settings);
        self.send_packet(&packet).await
    }

    /// Sets this clients access key within the DB Server. The server will persist the key until the session is disconnected, or connection is lost.
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// // sets the access key of the given client
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// ```
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn set_access_key(
        &mut self,
        key: String,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_key(key);
        self.send_packet(&packet)
    }

    /// Sets this clients access key within the DB Server. The server will persist the key until the session is disconnected, or connection is lost.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn set_access_key(
        &mut self,
        key: String,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_set_key(key);
        self.send_packet(&packet).await
    }

    /// Sends a packet to the clients currently connected database and returns the result
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub(crate) fn send_packet(
        &mut self,
        sent_packet: &DBPacket,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];

        // branch depending on if we are using encryption with communication
        let ser_packet = match &mut self.encryption {
            None => {
                let p = sent_packet
                    .serialize_packet()
                    .map_err(|err| PacketSerializationError(Error::from(err)));

                match p.as_ref() {
                    Ok(_) => {
                        info!("Successfully serialized packet");
                    }
                    Err(e) => {
                        error!("Failed to serialize packet: {:?}", e);
                    }
                }

                p?
            }
            Some(client_encrypt) => {
                // if we are sending a public key packet, we don't encrypt it, since the server needs this to send data back properly
                if !matches!(sent_packet, DBPacket::PubKey(_)) {
                    let p = client_encrypt
                        .encrypt_packet(sent_packet)
                        .map_err(PacketEncryptionError)?
                        .serialize_packet()
                        .map_err(|err| PacketSerializationError(Error::from(err)));

                    match p.as_ref() {
                        Ok(_) => {
                            info!("Successfully encrypted packet");
                        }
                        Err(e) => {
                            error!("Failed to encrypt packet: {:?}", e);
                        }
                    }

                    p?
                } else {
                    let p = sent_packet
                        .serialize_packet()
                        .map_err(|err| PacketSerializationError(Error::from(err)));

                    match p.as_ref() {
                        Ok(_) => {
                            info!("Successfully serialized public key packet");
                        }
                        Err(e) => {
                            error!("Failed to serialize public key packet: {:?}", e);
                        }
                    }

                    p?
                }
            }
        };

        let s_res = self
            .socket
            .write(ser_packet.as_bytes())
            .map_err(SocketWriteError);

        match s_res.as_ref() {
            Ok(len) => {
                info!("Successfully wrote {len} bytes to socket: {}", ser_packet);
            }
            Err(e) => {
                error!("Failed to write packet to socket: {:?}", e);
            }
        }

        s_res?;

        let read_len_res = self.socket.read(&mut buf).map_err(SocketReadError);

        match read_len_res.as_ref() {
            Ok(len) => {
                info!("Successfully read {len} bytes from socket");
            }
            Err(e) => {
                error!("Failed to read packet from socket: {:?}", e);
            }
        }

        let read_len = read_len_res?;

        match serde_json::from_slice::<Result<DBSuccessResponse<String>, DBPacketResponseError>>(
            &buf[0..read_len],
        ) {
            Ok(thing) => {
                match thing.as_ref() {
                    Ok(response) => {
                        info!("Successful response from server: {}", response);
                    }
                    Err(err) => {
                        error!("Error response from server: {}", err);
                    }
                }
                thing.map_err(DBResponseError)
            }
            Err(err) => {
                // if we fail to read a packet, check if it is an encrypted packet
                if let Some(client_private_key) = &self.encryption {
                    match client_private_key
                        .decrypt_server_packet(&buf[0..read_len])
                        .map_err(PacketEncryptionError)
                    {
                        Ok(decrypted) => {
                            info!("Successfully decrypted data from server packet");
                            match decrypted.as_ref() {
                                Ok(response) => {
                                    info!("Successful response from server: {}", response);
                                }
                                Err(err) => {
                                    error!("Error response from server: {}", err);
                                }
                            }
                            decrypted.map_err(DBResponseError)
                        }
                        Err(err) => {
                            error!("Error decrypting server packet: {:?}", err);
                            return Err(err);
                        }
                    }
                } else {
                    error!("Packet deserialization error: {}", err);
                    Err(PacketDeserializationError(Error::from(err)))
                }
            }
        }
    }

    /// Sends a packet to the clients currently connected database and returns the result
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub(crate) async fn send_packet(
        &mut self,
        sent_packet: &DBPacket,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let mut buf: [u8; 1024] = [0; 1024];

        // branch depending on if we are using encryption with communication
        let ser_packet = match &mut self.encryption {
            None => {
                let p = sent_packet
                    .serialize_packet()
                    .map_err(|err| PacketSerializationError(Error::from(err)));

                match p.as_ref() {
                    Ok(_) => {
                        info!("Successfully serialized packet");
                    }
                    Err(e) => {
                        error!("Failed to serialize packet: {:?}", e);
                    }
                }

                p?
            }
            Some(client_encrypt) => {
                // if we are sending a public key packet, we don't encrypt it, since the server needs this to send data back properly
                if !matches!(sent_packet, DBPacket::PubKey(_)) {
                    let p = client_encrypt
                        .encrypt_packet(sent_packet)
                        .map_err(PacketEncryptionError)?
                        .serialize_packet()
                        .map_err(|err| PacketSerializationError(Error::from(err)));

                    match p.as_ref() {
                        Ok(_) => {
                            info!("Successfully encrypted packet");
                        }
                        Err(e) => {
                            error!("Failed to encrypt packet: {:?}", e);
                        }
                    }

                    p?
                } else {
                    let p = sent_packet
                        .serialize_packet()
                        .map_err(|err| PacketSerializationError(Error::from(err)));

                    match p.as_ref() {
                        Ok(_) => {
                            info!("Successfully serialized public key packet");
                        }
                        Err(e) => {
                            error!("Failed to serialize public key packet: {:?}", e);
                        }
                    }

                    p?
                }
            }
        };

        let s_res = self
            .socket
            .write(ser_packet.as_bytes())
            .await
            .map_err(SocketWriteError);

        match s_res.as_ref() {
            Ok(len) => {
                info!("Successfully wrote {len} bytes to socket");
            }
            Err(e) => {
                error!("Failed to write packet to socket: {:?}", e);
            }
        }

        s_res?;

        let read_len_res = self.socket.read(&mut buf).await.map_err(SocketReadError);

        match read_len_res.as_ref() {
            Ok(len) => {
                info!("Successfully read {len} bytes from socket");
            }
            Err(e) => {
                error!("Failed to read packet from socket: {:?}", e);
            }
        }

        let read_len = read_len_res?;

        match serde_json::from_slice::<Result<DBSuccessResponse<String>, DBPacketResponseError>>(
            &buf[0..read_len],
        ) {
            Ok(thing) => {
                match thing.as_ref() {
                    Ok(response) => {
                        info!("Successful response from server: {}", response);
                    }
                    Err(err) => {
                        error!("Error response from server: {}", err);
                    }
                }
                thing.map_err(DBResponseError)
            }
            Err(err) => {
                // if we fail to read a packet, check if it is an encrypted packet
                if let Some(client_private_key) = &self.encryption {
                    match client_private_key
                        .decrypt_server_packet(&buf[0..read_len])
                        .map_err(PacketEncryptionError)
                    {
                        Ok(decrypted) => {
                            info!("Successfully decrypted data from server packet");
                            match decrypted.as_ref() {
                                Ok(response) => {
                                    info!("Successful response from server: {}", response);
                                }
                                Err(err) => {
                                    error!("Error response from server: {}", err);
                                }
                            }
                            decrypted.map_err(DBResponseError)
                        }
                        Err(err) => {
                            error!("Error decrypting server packet: {:?}", err);
                            return Err(err);
                        }
                    }
                } else {
                    error!("Packet deserialization error: {}", err);
                    Err(PacketDeserializationError(Error::from(err)))
                }
            }
        }
    }

    /// Creates a db through the client with the given name.
    /// Error on IO Error, or when the user lacks permissions to create a DB
    /// ```
    /// use std::time::Duration;
    /// use smol_db_client::prelude::SmolDbClient;
    /// use smol_db_common::db_packets::db_settings::DBSettings;
    ///
    /// let mut client = SmolDbClient::new("localhost:8222").unwrap();
    ///
    /// let _ = client.set_access_key("test_key_123".to_string()).unwrap();
    /// let _ = client.create_db("doctest_create_db",DBSettings::default()).unwrap();
    ///
    /// let _ = client.delete_db("doctest_create_db").unwrap();
    /// ```
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn create_db(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_create_db(db_name, db_settings);
        let resp = self.send_packet(&packet)?;

        Ok(resp)
    }

    /// Creates a db through the client with the given name.
    /// Error on IO Error, or when the user lacks permissions to create a DB
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn create_db(
        &mut self,
        db_name: &str,
        db_settings: DBSettings,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_create_db(db_name, db_settings);
        let resp = self.send_packet(&packet).await?;

        Ok(resp)
    }

    /// Writes to a db at the location specified, with the data given as a string.
    /// Returns the data in the location that was overwritten if there was any.
    /// Requires permissions to write to the given DB
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn write_db(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_write(db_name, db_location, data);

        self.send_packet(&packet)
    }

    /// Writes to a db at the location specified, with the data given as a string.
    /// Returns the data in the location that was overwritten if there was any.
    /// Requires permissions to write to the given DB
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn write_db(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_write(db_name, db_location, data);

        self.send_packet(&packet).await
    }

    /// Reads from a db at the location specific.
    /// Returns an error if there is no data in the location.
    /// Requires permissions to read from the given DB
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn read_db(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_read(db_name, db_location);

        self.send_packet(&packet)
    }

    /// Reads from a db at the location specific.
    /// Returns an error if there is no data in the location.
    /// Requires permissions to read from the given DB
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn read_db(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_read(db_name, db_location);

        self.send_packet(&packet).await
    }

    /// Deletes the given db by name.
    /// Requires super admin privileges on the given DB Server
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn delete_db(&mut self, db_name: &str) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_db(db_name);

        self.send_packet(&packet)
    }

    /// Deletes the given db by name.
    /// Requires super admin privileges on the given DB Server
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn delete_db(
        &mut self,
        db_name: &str,
    ) -> Result<DBSuccessResponse<String>, ClientError> {
        let packet = DBPacket::new_delete_db(db_name);

        self.send_packet(&packet).await
    }

    /// Lists all the current databases available by name from the server
    /// Only error on IO Error
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn list_db(&mut self) -> Result<Vec<DBPacketInfo>, ClientError> {
        let packet = DBPacket::new_list_db();

        let response = self.send_packet(&packet)?;

        match response {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<Vec<DBPacketInfo>>(&data) {
                Ok(thing) => Ok(thing),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Lists all the current databases available by name from the server
    /// Only error on IO Error
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn list_db(&mut self) -> Result<Vec<DBPacketInfo>, ClientError> {
        let packet = DBPacket::new_list_db();

        let response = self.send_packet(&packet).await?;

        match response {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<Vec<DBPacketInfo>>(&data) {
                Ok(thing) => Ok(thing),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Get the hashmap of the contents of a database. Contents are always String:String for the hashmap.
    /// Requires list permissions on the given DB
    /// ```
    /// use smol_db_client::prelude::SmolDbClient;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
    pub fn list_db_contents(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, String>, ClientError> {
        let packet = DBPacket::new_list_db_contents(db_name);

        let response = self.send_packet(&packet)?;

        match response {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<HashMap<String, String>>(&data) {
                Ok(thing) => Ok(thing),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Get the hashmap of the contents of a database. Contents are always String:String for the hashmap.
    /// Requires list permissions on the given DB
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn list_db_contents(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, String>, ClientError> {
        let packet = DBPacket::new_list_db_contents(db_name);

        let response = self.send_packet(&packet).await?;

        match response {
            SuccessNoData => Err(BadPacket),
            SuccessReply(data) => match serde_json::from_str::<HashMap<String, String>>(&data) {
                Ok(thing) => Ok(thing),
                Err(err) => Err(PacketDeserializationError(Error::from(err))),
            },
        }
    }

    /// Lists the given db's contents, deserializing the contents into a hash map.
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
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

    /// Lists the given db's contents, deserializing the contents into a hash map.
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn list_db_contents_generic<T>(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        let contents = self.list_db_contents(db_name).await?;
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
    #[cfg(not(feature = "async"))]
    #[tracing::instrument(skip(data))]
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
                    SuccessNoData => Ok(smol_db_common::prelude::SuccessNoData),
                    SuccessReply(data_string) => match serde_json::from_str::<T>(&data_string) {
                        Ok(thing) => Ok(SuccessReply(thing)),
                        Err(err) => Err(PacketDeserializationError(Error::from(err))),
                    },
                },
                Err(err) => Err(err),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        }
    }

    /// Writes to the db while serializing the given data, returning the data at the location given and deserialized to the same type.
    #[cfg(feature = "async")]
    #[tracing::instrument(skip(data))]
    pub async fn write_db_generic<T>(
        &mut self,
        db_name: &str,
        db_location: &str,
        data: T,
    ) -> Result<DBSuccessResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match serde_json::to_string(&data) {
            Ok(ser_data) => match self.write_db(db_name, db_location, &ser_data).await {
                Ok(response) => match response {
                    SuccessNoData => Ok(smol_db_common::prelude::SuccessNoData),
                    SuccessReply(data_string) => match serde_json::from_str::<T>(&data_string) {
                        Ok(thing) => Ok(SuccessReply(thing)),
                        Err(err) => Err(PacketDeserializationError(Error::from(err))),
                    },
                },
                Err(err) => Err(err),
            },
            Err(err) => Err(PacketSerializationError(Error::from(err))),
        }
    }

    /// Reads from db and tries to deserialize the content at the location to the given generic
    #[cfg(not(feature = "async"))]
    #[tracing::instrument]
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
                SuccessNoData => Ok(SuccessNoData),
                SuccessReply(read_data) => match serde_json::from_str::<T>(&read_data) {
                    Ok(data) => Ok(SuccessReply(data)),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                },
            },
            Err(err) => Err(err),
        }
    }

    /// Reads from db and tries to deserialize the content at the location to the given generic
    #[cfg(feature = "async")]
    #[tracing::instrument]
    pub async fn read_db_generic<T>(
        &mut self,
        db_name: &str,
        db_location: &str,
    ) -> Result<DBSuccessResponse<T>, ClientError>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        match self.read_db(db_name, db_location).await {
            Ok(data) => match data {
                SuccessNoData => Ok(smol_db_common::prelude::SuccessNoData),
                SuccessReply(read_data) => match serde_json::from_str::<T>(&read_data) {
                    Ok(data) => Ok(SuccessReply(data)),
                    Err(err) => Err(PacketDeserializationError(Error::from(err))),
                },
            },
            Err(err) => Err(err),
        }
    }
}
