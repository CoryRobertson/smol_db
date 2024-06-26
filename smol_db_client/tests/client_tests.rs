#[cfg(test)]
#[allow(unused_imports, clippy::bool_assert_comparison)]
#[cfg(not(feature = "async"))]
mod tests {
    use serde::{Deserialize, Serialize};
    use smol_db_client::prelude::*;
    use std::fs::read;
    use std::thread;
    use std::time::Duration;
    use tracing::debug;

    #[test]
    fn test_stream() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);
        let create_response = client
            .create_db("stream_test", DBSettings::default())
            .unwrap();
        assert_eq!(create_response, SuccessNoData);

        for i in 0..10 {
            let data = format!("{i}");
            client
                .write_db("stream_test", data.as_str(), data.as_str())
                .unwrap();
        }

        let table_iter = client.stream_table("stream_test").unwrap();

        let list = table_iter.collect::<Vec<(String, String)>>();

        assert_eq!(list.len(), 10);

        for i in 0..10 {
            assert!(list.contains(&(i.to_string(), i.to_string())));
        }

        let delete_response = client.delete_db("stream_test").unwrap();
        assert_eq!(delete_response, SuccessNoData);
    }

    #[test]
    fn test_client() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let create_response = client.create_db("test2", DBSettings::default()).unwrap();
        assert_eq!(create_response, SuccessNoData);

        let data = "this_is_data";
        let write_response = client.write_db("test2", "location1", data).unwrap();
        assert_eq!(write_response, SuccessNoData);

        let read_response = client.read_db("test2", "location1").unwrap();

        match read_response {
            SuccessReply(response_data) => {
                assert_eq!(&response_data, data);
            }
            SuccessNoData => {
                panic!("data response was not as expected");
            }
        }

        let data2 = "this_is_not_data";
        let write_response2 = client.write_db("test2", "location1", data2).unwrap();

        match write_response2 {
            SuccessReply(previous_data) => {
                assert_eq!(data, &previous_data)
            }
            _ => {
                panic!("Write db 2 failed.")
            }
        }

        let read_response2 = client.read_db("test2", "location1").unwrap();

        match read_response2 {
            SuccessReply(response_data) => {
                assert_eq!(&response_data, data2);
            }
            _ => {
                panic!("data response was not as expected");
            }
        }

        let delete_response = client.delete_db("test2").unwrap();
        assert_eq!(delete_response, SuccessNoData);
    }

    #[derive(PartialEq, Eq, Deserialize, Serialize, Clone, Debug)]
    struct TestStruct {
        a: u32,
        b: bool,
        c: i32,
        d: String,
    }

    #[test]
    fn test_missing_create_db_permissions() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let resp = client
            .create_db("not enough permissions", DBSettings::default())
            .unwrap_err();

        match resp {
            DBResponseError(resp) => {
                assert_eq!(resp, InvalidPermissions);
            }
            _ => {
                unreachable!()
            }
        }
    }

    #[test]
    fn test_generics_client() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

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
        assert_eq!(create_db_response, SuccessNoData);

        let write_db_response1 = client
            .write_db_generic("test_generics", "location1", test_data1.clone())
            .unwrap();
        assert_eq!(write_db_response1, SuccessNoData);

        let read_db_response1 = client
            .read_db_generic::<TestStruct>("test_generics", "location1")
            .unwrap();

        match read_db_response1 {
            SuccessReply(received_struct) => {
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
            SuccessReply(previous_struct) => {
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
            SuccessReply(received_struct) => {
                assert_eq!(received_struct, test_data2);
            }
            _ => {
                panic!("Read db error 1")
            }
        }

        let delete_db_response = client.delete_db("test_generics").unwrap();
        assert_eq!(delete_db_response, SuccessNoData);
    }

    #[test]
    fn test_list_db() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let create_db_response1 = client
            .create_db("test_db_1", DBSettings::default())
            .unwrap();
        assert_eq!(create_db_response1, SuccessNoData);

        let create_db_response2 = client
            .create_db("test_db_2", DBSettings::default())
            .unwrap();
        assert_eq!(create_db_response2, SuccessNoData);

        let list_db_response = client.list_db().unwrap();

        assert!(list_db_response.clone().len() >= 2);

        assert!(list_db_response.contains(&DBPacketInfo::new("test_db_1")));
        assert!(list_db_response.contains(&DBPacketInfo::new("test_db_2")));

        let delete_db_response1 = client.delete_db("test_db_1").unwrap();
        assert_eq!(delete_db_response1, SuccessNoData);

        let delete_db_response2 = client.delete_db("test_db_2").unwrap();
        assert_eq!(delete_db_response2, SuccessNoData);
    }

    #[test]
    #[cfg(feature = "statistics")]
    fn test_get_stats() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let create_db_response1 = client
            .create_db("test_db_stats", DBSettings::default())
            .unwrap();
        assert_eq!(create_db_response1, SuccessNoData);

        {
            let r = client.get_stats("test_db_stats").unwrap();
            assert_eq!(r.get_total_req(), 1);
        }

        {
            let list = client.list_db_contents("test_db_stats").unwrap();
            assert_eq!(list.len(), 0);
        }

        {
            let r = client.get_stats("test_db_stats").unwrap();
            assert_eq!(r.get_total_req(), 3);
        }

        {
            let r = client.get_stats("test_db_stats").unwrap();
            assert_eq!(r.get_total_req(), 4);
        }

        {
            let delete_response = client.delete_db("test_db_stats").unwrap();
            assert_eq!(delete_response, SuccessNoData);
        }
    }

    // #[test]
    // fn test_empty_db_list() {
    //     let mut client = SmolDbClient::new("localhost:8222").unwrap();
    //
    //     let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
    //     assert_eq!(set_key_response, SuccessNoData);
    //
    //     let mut count = 0;
    //     loop {
    //         // continue looping indefinitely until we manage to read an empty list db, verifying that serialization works even when the list would be empty.
    //         let list = client.list_db().unwrap();
    //         let len = list.len();
    //         // lazy way to check
    //         thread::sleep(Duration::from_millis(250)); // wait a small amount of time between lists so we dont dominate the thread pool on the server.
    //         if len == 0 {
    //             // if we find a 0 length return, then we have clearly not panicked and can stop looping, allowing the test to be successful
    //             break;
    //         }
    //         if count >= 64 {
    //             // allow 16* 250ms = 16 seconds to pass before declaring the test a failure
    //             panic!("count not read empty db list within reasonable amount of time, its possible there were databases stored that were not related to the unit tests.")
    //         }
    //         count += 1;
    //     }
    // }

    #[test]
    fn test_list_db_contents() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let db_name = "test_db_contents1";

        let create_db_response1 = client.create_db(db_name, DBSettings::default()).unwrap();
        assert_eq!(create_db_response1, SuccessNoData);

        let write_response1 = client.write_db(db_name, "location1", "123").unwrap();
        assert_eq!(write_response1, SuccessNoData);

        let write_response2 = client.write_db(db_name, "location2", "456").unwrap();
        assert_eq!(write_response2, SuccessNoData);

        let list_db_contents_response = client.list_db_contents(db_name).unwrap();

        assert_eq!(list_db_contents_response.get("location1").unwrap(), "123");
        assert_eq!(list_db_contents_response.get("location2").unwrap(), "456");

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, SuccessNoData);
    }

    #[test]
    fn test_list_db_contents_empty() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let db_name = "test_db_contents_empty1";

        let create_db_response1 = client.create_db(db_name, DBSettings::default()).unwrap();
        assert_eq!(create_db_response1, SuccessNoData);

        let contents = client.list_db_contents(db_name).unwrap();

        assert_eq!(contents.is_empty(), true); // contents should be empty as no write operations occurred.

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, SuccessNoData);
    }

    #[test]
    fn test_list_db_contents_generic() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

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
        assert_eq!(create_response, SuccessNoData);

        let write_response1 = client
            .write_db_generic(db_name, "location1", test_data1.clone())
            .unwrap();
        assert_eq!(write_response1, SuccessNoData);

        let write_response2 = client
            .write_db_generic(db_name, "location2", test_data2.clone())
            .unwrap();
        assert_eq!(write_response2, SuccessNoData);

        let list = client
            .list_db_contents_generic::<TestStruct>(db_name)
            .unwrap();

        assert_eq!(list.len(), 2);

        assert_eq!(list.get("location1").unwrap().clone(), test_data1);
        assert_eq!(list.get("location2").unwrap().clone(), test_data2);

        let delete_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_response, SuccessNoData);
    }

    #[test]
    fn test_get_db_settings() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();
        let db_settings_test = DBSettings::new(
            Duration::from_secs(29),
            (false, true, false),
            (true, false, true),
            vec![],
            vec![],
        );
        let db_name = "test_getdb_settings";

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, SuccessNoData);

        let get_settings = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings, db_settings_test.clone());

        let received_settings = get_settings;
        assert_eq!(received_settings, db_settings_test);

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, SuccessNoData);
    }

    #[test]
    fn test_set_db_settings() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();
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
        assert_eq!(set_key_response, SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, SuccessNoData);

        let get_settings = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings, db_settings_test.clone());

        let received_settings = get_settings;
        assert_eq!(received_settings, db_settings_test.clone());
        assert_ne!(received_settings, new_db_settings_test.clone());

        let set_settings_response = client
            .set_db_settings(db_name, new_db_settings_test.clone())
            .unwrap();
        assert_eq!(set_settings_response, SuccessNoData);

        let get_settings2 = client.get_db_settings(db_name).unwrap();
        assert_eq!(get_settings2, new_db_settings_test.clone());

        let received_settings2 = get_settings2;
        assert_eq!(received_settings2, new_db_settings_test.clone());
        assert_ne!(received_settings2, db_settings_test.clone());

        let delete_db_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_db_response, SuccessNoData);
    }

    #[test]
    fn test_get_role() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();
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
        assert_eq!(set_key_response, SuccessNoData);

        let create_response = client.create_db(db_name, db_settings_test.clone()).unwrap();
        assert_eq!(create_response, SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), SuperAdmin);
        // set admin key
        let set_key_response = client.set_access_key(admin_key.clone()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), Admin);
        // set user key
        let set_key_response = client.set_access_key(user_key.clone()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), User);

        // set other key
        let set_key_response = client.set_access_key(other_key.clone()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        assert_eq!(client.get_role(db_name).unwrap(), Other);

        let set_key_response = client.set_access_key("test_key_123".to_string()).unwrap();
        assert_eq!(set_key_response, SuccessNoData);

        let delete_response = client.delete_db(db_name).unwrap();
        assert_eq!(delete_response, SuccessNoData);
    }

    #[test]
    fn test_delete_data() {
        let mut client = SmolDbClient::new("localhost:8222").unwrap();
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
            let read_response = client.read_db(db_name, db_location);
            assert_eq!(read_response.unwrap_err(), DBResponseError(ValueNotFound));
        }

        {
            let delete_data_response = client.delete_data(db_name, db_location);
            assert_eq!(
                delete_data_response.unwrap_err(),
                DBResponseError(ValueNotFound)
            );
        }

        {
            let delete_response = client.delete_db(db_name).unwrap();
            assert_eq!(delete_response, SuccessNoData);
        }
    }
}
