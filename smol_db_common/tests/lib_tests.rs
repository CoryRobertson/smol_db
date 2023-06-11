#[cfg(test)]
mod tests {
    use smol_db_common::db_data::DBData;
    use smol_db_common::db_list::DBList;
    use smol_db_common::db_packets::db_location::DBLocation;
    use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    use smol_db_common::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
    use smol_db_common::db_packets::db_settings::DBSettings;
    use std::fs;
    use std::path::Path;

    #[test]
    /// Tests the following:
    /// DB creation and file existing
    /// DB reading with value not present
    /// DB writing to empty key
    /// DB reading with value present
    /// DB writing to present value
    /// DB deletion and file removal
    /// DB deletion with db not existing
    fn full_db_test() {
        let db_list = DBList::load_db_list();
        let db_path = Path::new("./test_db");
        let db_name = "test_db";
        let db_packet = DBPacketInfo::new(db_name);
        let data_location = DBLocation::new("test");
        let db_data1 = DBData::new("test_data123".to_string());
        let db_data2 = DBData::new("123test_data".to_string());
        let super_admin_key = "test_key_123".to_string();

        assert!(!db_path.exists()); // verify the db is not already there

        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(super_admin_key.clone());

        db_list.create_db(db_name, DBSettings::default(), &super_admin_key);

        assert!(db_path.exists()); // verify the db exists after creation

        let resp_value_not_found = db_list.read_db(&db_packet, &data_location, &super_admin_key);
        assert!(
            resp_value_not_found == DBPacketResponse::Error(DBPacketResponseError::ValueNotFound)
        );

        let resp_write_value = db_list.write_db(
            &db_packet,
            &data_location,
            db_data1.clone(),
            &super_admin_key,
        );

        assert!(resp_write_value == DBPacketResponse::SuccessNoData);

        let resp_value_read = db_list.read_db(&db_packet, &data_location, &super_admin_key);

        assert_eq!(
            resp_value_read,
            DBPacketResponse::SuccessReply(db_data1.get_data().to_string())
        );

        let resp_value_write2 = db_list.write_db(
            &db_packet,
            &data_location,
            db_data2.clone(),
            &super_admin_key,
        );

        assert_eq!(
            resp_value_write2,
            DBPacketResponse::SuccessReply(db_data1.get_data().to_string())
        );

        let resp_delete_db = db_list.delete_db(db_name, &super_admin_key);

        assert_eq!(resp_delete_db, DBPacketResponse::SuccessNoData);

        let resp_delete_db2 = db_list.delete_db(db_name, &super_admin_key);

        assert_eq!(
            resp_delete_db2,
            DBPacketResponse::Error(DBPacketResponseError::DBNotFound)
        );
    }

    #[test]
    /// Tests the following:
    /// Successful DB Creation
    /// Failed DB creation
    /// Reading from non-existent DB
    fn test_errors() {
        let db_list = DBList::load_db_list();
        let db_path = Path::new("./test_db2");
        let db_name = "test_db2";
        let data_location = DBLocation::new("test");

        let super_admin_key = "test_key_123".to_string();

        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(super_admin_key.clone());

        assert!(!db_path.exists()); // verify the db is not already there

        let expected = DBPacketResponse::SuccessNoData;
        let successful_db_creation =
            db_list.create_db(db_name, DBSettings::default(), &super_admin_key);
        assert_eq!(successful_db_creation, expected);

        let expected = DBPacketResponse::Error(DBPacketResponseError::DBAlreadyExists);
        let failed_db_creation =
            db_list.create_db(db_name, DBSettings::default(), &super_admin_key);
        assert_eq!(failed_db_creation, expected);

        let expected = DBPacketResponse::Error(DBPacketResponseError::DBNotFound);
        let db_not_found_resp = db_list.read_db(
            &DBPacketInfo::new("not_a_real_db"),
            &data_location,
            &super_admin_key,
        );
        assert_eq!(db_not_found_resp, expected);

        fs::remove_file(db_path).unwrap(); // clean up tests
    }

    #[test]
    /// Tests uncached database reads
    fn test_cache_miss() {
        let db_list = DBList::load_db_list();
        let db_path = Path::new("./test_db3");
        let db_name = "test_db3";
        let data_location = DBLocation::new("test");
        let db_packet = DBPacketInfo::new(db_name);
        let db_data1 = DBData::new("test_data123".to_string());
        let db_data2 = DBData::new("123test_data".to_string());
        let super_admin_key = "test_key_123".to_string();

        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(super_admin_key.clone());

        assert!(!db_path.exists()); // verify the db is not already there
        assert!(!Path::new("./db_list.ser").exists()); // verify the db is not already there

        let expected = DBPacketResponse::SuccessNoData;
        let successful_db_creation =
            db_list.create_db(db_name, DBSettings::default(), &super_admin_key);
        assert_eq!(successful_db_creation, expected);

        let resp_write_value = db_list.write_db(
            &db_packet,
            &data_location,
            db_data1.clone(),
            &super_admin_key,
        );
        assert!(resp_write_value == DBPacketResponse::SuccessNoData);

        db_list.save_db_list();
        db_list.save_all_db();
        db_list.cache.write().unwrap().clear();

        assert_eq!(db_list.cache.read().unwrap().len(), 0);

        let resp_uncached = db_list.read_db(&db_packet, &data_location, &super_admin_key);
        assert_eq!(
            resp_uncached,
            DBPacketResponse::SuccessReply(db_data1.get_data().to_string())
        );

        db_list.save_db_list();
        db_list.save_all_db();
        db_list.cache.write().unwrap().clear();

        assert_eq!(db_list.cache.read().unwrap().len(), 0);

        let resp_write_value2 = db_list.write_db(
            &db_packet,
            &data_location,
            db_data2.clone(),
            &super_admin_key,
        );
        assert_eq!(
            resp_write_value2,
            DBPacketResponse::SuccessReply(db_data1.get_data().to_string())
        );

        fs::remove_file(db_path).unwrap(); // clean up tests
        fs::remove_file("db_list.ser").unwrap(); // clean up tests
    }
}
