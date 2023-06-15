#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use serde::{Deserialize, Serialize};
    use smol_db_client::Client;
    use smol_db_common::db::Role::{Admin, Other, SuperAdmin, User};
    use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    use smol_db_common::db_packets::db_packet_response::DBPacketResponse::{
        Error, SuccessNoData, SuccessReply,
    };
    use smol_db_common::db_packets::db_packet_response::DBPacketResponseError::ValueNotFound;
    use smol_db_common::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
    use smol_db_common::db_packets::db_settings::DBSettings;
    use std::fs::read;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_client() {
        let mut client = Client::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

        let mut count = 0;
        loop {
            // continue looping indefinitely until we manage to read an empty list db, verifying that serialization works even when the list would be empty.
            let list = client.list_db().unwrap();
            let len = list.len();
            // lazy way to check
            thread::sleep(Duration::from_millis(250)); // wait a small amount of time between lists so we dont dominate the thread pool on the server.
            if len == 0 {
                // if we find a 0 length return, then we have clearly not panicked and can stop looping, allowing the test to be successful
                break;
            }
            if count >= 16 {
                // allow 16* 250ms = 4 seconds to pass before declaring the test a failure
                panic!("count not read empty db list within reasonable amount of time.")
            }
            count += 1;
        }
    }

    #[test]
    fn test_list_db_contents() {
        let mut client = Client::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        match set_key_response {
            DBPacketResponse::SuccessNoData => {}
            _ => {
                panic!("Create db failed.");
            }
        }

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

    #[test]
    fn test_get_db_settings() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_settings_test = DBSettings::new(
            Duration::from_secs(29),
            (false, true, false),
            (true, false, true),
            vec![],
            vec![],
        );
        let db_name = "test_getdb_settings";

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, DBPacketResponse::SuccessNoData);

        let get_settings = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings, db_settings_test.clone());

        let received_settings = get_settings;
        assert_eq!(received_settings, db_settings_test);

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, DBPacketResponse::SuccessNoData);
    }

    #[test]
    fn test_set_db_settings() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_settings_test = DBSettings::new(
            Duration::from_secs(27),
            (false, true, true),
            (false, false, true),
            vec![],
            vec![],
        );
        let new_db_settings_test = DBSettings::new(
            Duration::from_secs(23),
            (false, false, true),
            (true, false, true),
            vec![],
            vec![],
        );
        let db_name = "test_setdb_settings";

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, DBPacketResponse::SuccessNoData);

        let get_settings = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings, db_settings_test.clone());

        let received_settings = get_settings;
        assert_eq!(received_settings, db_settings_test.clone());
        assert_ne!(received_settings, new_db_settings_test.clone());

        let set_settings_response = client
            .set_db_settings(db_name, new_db_settings_test.clone())
            .unwrap();
        assert_eq!(set_settings_response, DBPacketResponse::SuccessNoData);

        let get_settings2 = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings2, new_db_settings_test.clone());

        let received_settings2 = get_settings2;
        assert_eq!(received_settings2, new_db_settings_test.clone());
        assert_ne!(received_settings2, db_settings_test.clone());

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, DBPacketResponse::SuccessNoData);
    }

    #[test]
    fn test_get_role() {
        let mut client = Client::new("localhost:8222").unwrap();
        let user_key = "this is a user key that works".to_string();
        let admin_key = "this is an admin key that works".to_string();
        let other_key = "this is not an admin, super admin, or user key".to_string();
        let db_settings_test = DBSettings::new(
            Duration::from_secs(21),
            (false, true, false),
            (true, false, true),
            vec![admin_key.clone()],
            vec![user_key.clone()],
        );
        let db_name = "test_getrole";

        // set key to super admin key
        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, DBPacketResponse::SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), SuperAdmin);
        // set admin key
        let set_key_response = client.set_access_key(admin_key.clone()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), Admin);
        // set user key
        let set_key_response = client.set_access_key(user_key.clone()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), User);

        // set other key
        let set_key_response = client.set_access_key(other_key.clone()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), Other);

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, DBPacketResponse::SuccessNoData);

        let delete_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_response, DBPacketResponse::SuccessNoData);
    }

    #[test]
    fn test_delete_data() {
        let mut client = Client::new("localhost:8222").unwrap();
        let db_settings_test = DBSettings::new(
            Duration::from_secs(21),
            (false, true, false),
            (true, false, true),
            vec![],
            vec![],
        );
        let db_name = "test_delete_data";
        let db_location = "location1";
        let data = "super cool data";

        {
            // set key to super admin key
            let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
            assert_eq!(set_key_response, SuccessNoData);
        }

        {
            let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
            assert_eq!(create_response, SuccessNoData);
        }

        {
            let write_response = client.write_db(db_name, db_location, data).unwrap();
            assert_eq!(write_response, SuccessNoData);
        }

        {
            let read_response = client.read_db(db_name, db_location).unwrap();
            assert_eq!(read_response, SuccessReply(data.to_string()));
        }

        {
            let delete_response_data = client.delete_data(db_name, db_location).unwrap();
            assert_eq!(delete_response_data, SuccessReply(data.to_string()));
        }

        {
            let read_response = client.read_db(db_name, db_location).unwrap();
            assert_eq!(read_response, Error(ValueNotFound));
        }

        {
            let delete_data_response = client.delete_data(db_name, db_location).unwrap();
            assert_eq!(delete_data_response, Error(ValueNotFound));
        }

        {
            let delete_response = client.delete_db(db_name).unwrap();
            assert_eq!(delete_response, SuccessNoData);
        }
    }
}
