#[cfg(test)]
mod tests {
    use smol_db_common::db_list::DBResult;
    use smol_db_common::encryption::client_encrypt::ClientKey;
    use smol_db_common::encryption::server_encrypt::ServerKey;
    use smol_db_common::prelude::{DBPacket, DBSuccessResponse};

    #[test]
    fn test_server_key_usage() {
        let mut server_key = ServerKey::new().unwrap();
        let client_key = ClientKey::new(server_key.get_pub_key().clone()).unwrap();
        let reply_data = "cool reply!";
        let packet: DBResult = Ok(DBSuccessResponse::SuccessReply(reply_data.to_string()));
        let packet_str = serde_json::to_string(&packet).unwrap();
        let enc_data = server_key
            .encrypt_packet(&packet_str, client_key.get_pub_key())
            .unwrap();

        let decy_data = client_key
            .decrypt_server_packet(enc_data.get_data())
            .unwrap()
            .unwrap()
            .as_option()
            .unwrap()
            .to_string();
        assert_eq!(decy_data, reply_data);
    }

    #[test]
    fn test_client_key_usage() {
        let server_key = ServerKey::new().unwrap();
        let mut client_key = ClientKey::new(server_key.get_pub_key().clone()).unwrap();
        let db_name = "db 1";
        let write_location = "location 1";
        let reply_data = "woahhh!";
        let packet = DBPacket::new_write(db_name, write_location, reply_data);
        let ency_packet = client_key.encrypt_packet(&packet).unwrap();

        match ency_packet {
            DBPacket::Encrypted(ency) => {
                let decy = server_key.decrypt_client_packet(&ency).unwrap();

                match packet {
                    DBPacket::Write(a, b, c) => {
                        assert_eq!(a.get_db_name(), db_name);
                        assert_eq!(b.as_key(), write_location);
                        assert_eq!(c.get_data(), reply_data);
                    }
                    _ => {
                        panic!("Wrong packet used to test");
                    }
                }

                match decy {
                    DBPacket::Write(a, b, c) => {
                        assert_eq!(a.get_db_name(), db_name);
                        assert_eq!(b.as_key(), write_location);
                        assert_eq!(c.get_data(), reply_data);
                    }
                    _ => {
                        panic!("Wrong packet received from decryption");
                    }
                }
            }
            _ => {
                panic!("Incorrect packet made from encryption on client side");
            }
        }
    }
}
