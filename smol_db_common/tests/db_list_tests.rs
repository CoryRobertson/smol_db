#[cfg(test)]
#[allow(unused_imports, clippy::bool_assert_comparison)]
mod tests {

    use smol_db_common::prelude::*;
    use std::collections::HashMap;
    use std::fs::File;
    use std::hash::Hash;
    use std::path::PathBuf;
    use std::sync::RwLock;
    use std::time::Duration;
    use std::{fs, thread};

    static TEST_SUPER_ADMIN_KEY: &str = "test_admin_key";
    static TEST_USER_KEY: &str = "test_user_key";

    fn get_db_test_settings() -> DBSettings {
        DBSettings::new(
            Duration::from_secs(30),
            (false, false, false),
            (true, true, true),
            vec![TEST_SUPER_ADMIN_KEY.to_string()],
            vec![TEST_USER_KEY.to_string()],
        )
    }

    fn get_db_list_for_testing() -> DBList {
        DBList {
            list: RwLock::new(vec![]),
            cache: RwLock::new(HashMap::new()),
            super_admin_hash_list: RwLock::new(vec![]),
            server_key: Default::default(),
        }
    }

    #[test]
    fn test_is_super_admin() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(
            db_list.is_super_admin(&TEST_SUPER_ADMIN_KEY.to_string()),
            true
        );
        assert_eq!(
            db_list.is_super_admin(&"probably not an admin key".to_string()),
            false
        );
    }

    #[test]
    fn test_create_db() {
        let _ = fs::create_dir("./data");
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_create";
        let create_response = db_list
            .create_db(
                db_name,
                get_db_test_settings(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            )
            .unwrap();

        assert_eq!(create_response, SuccessNoData);

        let create_response_db_already_exists = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(
            create_response_db_already_exists.unwrap_err(),
            DBAlreadyExists
        );

        let create_response_db_invalid_perms = db_list.create_db(
            "other_db",
            get_db_test_settings(),
            &"this is not an admin key".to_string(),
        );

        assert_eq!(
            create_response_db_invalid_perms.unwrap_err(),
            InvalidPermissions
        );

        // clean up unit test files
        fs::remove_file("./data/test_dblist_1_create").unwrap();
    }

    #[test]
    fn test_delete_db() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_delete";

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        let invalid_perms_delete_response =
            db_list.delete_db(db_name, &"not a working admin key".to_string());
        assert_eq!(
            invalid_perms_delete_response.unwrap_err(),
            InvalidPermissions
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);

        if let Ok(f) = File::open(PathBuf::from("./data").join(db_name)) {
            panic!("db not deleted {:?}", f)
        }

        let delete_response_not_listed =
            db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response_not_listed.unwrap_err(), DBNotFound);
    }

    #[test]
    fn test_write_and_read_db() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_read_write";
        let db_pack_info = DBPacketInfo::new(db_name);
        let db_location = DBLocation::new("location1");
        let db_data = DBData::new("this is data".to_string());

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        let write_invalid_perms = db_list.write_db(
            &db_pack_info,
            &db_location,
            &db_data.clone(),
            &"not a working client key".to_string(),
        );
        assert_eq!(write_invalid_perms.unwrap_err(), InvalidPermissions);

        let write_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            &db_data.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(write_response.unwrap(), SuccessNoData);

        let write_response2 = db_list
            .write_db(
                &db_pack_info,
                &db_location,
                &db_data.clone(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            )
            .unwrap();

        match write_response2 {
            SuccessNoData => {
                panic!("Bad response from write db");
            }
            SuccessReply(data) => {
                assert_eq!(data, db_data.get_data().to_string());
            }
        }

        let read_response = db_list
            .read_db(
                &db_pack_info,
                &db_location,
                &TEST_SUPER_ADMIN_KEY.to_string(),
            )
            .unwrap();
        match read_response {
            SuccessNoData => {
                panic!("No data read from location");
            }
            SuccessReply(data) => {
                assert_eq!(data, db_data.get_data().to_string());
            }
        }

        let read_user_perms_response = db_list
            .read_db(&db_pack_info, &db_location, &TEST_USER_KEY.to_string())
            .unwrap();
        match read_user_perms_response {
            SuccessNoData => {
                panic!("Unable to read with user perms");
            }
            SuccessReply(data) => {
                assert_eq!(data, db_data.get_data().to_string());
            }
        }

        let read_invalid_perms_response = db_list
            .read_db(
                &db_pack_info,
                &db_location,
                &"not a user key or an admin key".to_string(),
            )
            .unwrap_err();
        assert_eq!(read_invalid_perms_response, InvalidPermissions);

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_add_and_remove_user() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_add_remove_user";
        let db_pack_info = DBPacketInfo::new(db_name);
        let db_location = DBLocation::new("location1");
        let db_data = DBData::new("this is data".to_string());
        let new_user_key = "new user key that gets added".to_string();

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        // add user without perms, and with perms, and the test users key
        let add_user_invalid_perms1 = db_list
            .add_user(
                &db_pack_info,
                new_user_key.clone(),
                &TEST_USER_KEY.to_string(),
            )
            .unwrap_err();
        assert_eq!(add_user_invalid_perms1, InvalidPermissions);
        let add_user_invalid_perms2 = db_list.add_user(
            &db_pack_info,
            new_user_key.clone(),
            &"not a working key".to_string(),
        );
        assert_eq!(add_user_invalid_perms2.unwrap_err(), InvalidPermissions);
        let add_user_response = db_list.add_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(add_user_response.unwrap(), SuccessNoData);

        // try writing data to the db with the perms of the new user
        let write_with_new_user_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            &db_data.clone(),
            &new_user_key.to_string(),
        );
        assert_eq!(write_with_new_user_response.unwrap(), SuccessNoData);
        let read_with_new_user_response =
            db_list.read_db(&db_pack_info, &db_location, &new_user_key.to_string());
        match read_with_new_user_response.unwrap() {
            SuccessNoData => {
                panic!("No data read from read with new user");
            }
            SuccessReply(data) => {
                assert_eq!(data, db_data.clone().get_data().to_string());
            }
        }

        // remove user with invalid perms, then eventually remove the user with an admin perm, and try removing the user again and note that the user is not found
        let remove_user_invalid_perms1 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone().as_str(),
            &TEST_USER_KEY.to_string(),
        );
        assert_eq!(remove_user_invalid_perms1.unwrap_err(), InvalidPermissions);
        let remove_user_invalid_perms2 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone().as_str(),
            &"not a working key".to_string(),
        );
        assert_eq!(remove_user_invalid_perms2.unwrap_err(), InvalidPermissions);
        let remove_user_response1 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone().as_str(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(remove_user_response1.unwrap(), SuccessNoData);
        let remove_user_response2 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone().as_str(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(remove_user_response2.unwrap_err(), UserNotFound);

        // write to the db with invalid perms of the added user, who was removed, also attempt to read using the removed users key
        let write_with_new_user_response2 = db_list.write_db(
            &db_pack_info,
            &db_location,
            &db_data.clone(),
            &new_user_key.to_string(),
        );
        assert_eq!(
            write_with_new_user_response2.unwrap_err(),
            InvalidPermissions
        );
        let read_with_new_user_response2 =
            db_list.read_db(&db_pack_info, &db_location, &new_user_key.to_string());
        assert_eq!(
            read_with_new_user_response2.unwrap_err(),
            InvalidPermissions
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_add_and_remove_admin() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_add_remove_admin";
        let db_pack_info = DBPacketInfo::new(db_name);
        let new_admin_key = "new admin key that gets added".to_string();
        let new_user_key = "new user key that gets added".to_string();

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        let add_admin_without_perms1 = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &"this is not a working key".to_string(),
        );
        assert_eq!(add_admin_without_perms1.unwrap_err(), InvalidPermissions);
        let add_admin_without_perms2 = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &TEST_USER_KEY.to_string(),
        );
        assert_eq!(add_admin_without_perms2.unwrap_err(), InvalidPermissions);
        let add_admin_with_perms = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(add_admin_with_perms.unwrap(), SuccessNoData);

        let new_admin_add_user =
            db_list.add_user(&db_pack_info, new_user_key.clone(), &new_admin_key.clone());
        assert_eq!(new_admin_add_user.unwrap(), SuccessNoData);

        let remove_admin_without_perms1 = db_list.remove_admin(
            &db_pack_info,
            new_admin_key.clone().as_str(),
            &"this is not a working key".to_string(),
        );
        assert_eq!(remove_admin_without_perms1.unwrap_err(), InvalidPermissions);
        let remove_admin_without_perms2 = db_list.remove_admin(
            &db_pack_info,
            new_admin_key.clone().as_str(),
            &new_admin_key.clone(),
        );
        assert_eq!(remove_admin_without_perms2.unwrap_err(), InvalidPermissions);
        let remove_admin_success_response = db_list.remove_admin(
            &db_pack_info,
            new_admin_key.clone().as_str(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(remove_admin_success_response.unwrap(), SuccessNoData);

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_list_db() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_db_list1";

        {
            let db_list_response = db_list.list_db();
            match db_list_response.unwrap() {
                SuccessNoData => {
                    panic!("Unexpected db response");
                }
                SuccessReply(data) => {
                    let v = serde_json::from_str::<Vec<DBPacketInfo>>(&data).unwrap();
                    assert_eq!(v.len(), 0);
                }
            }
        }

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        {
            let db_list_response = db_list.list_db();
            match db_list_response.unwrap() {
                SuccessNoData => {}
                SuccessReply(data) => {
                    let v = serde_json::from_str::<Vec<DBPacketInfo>>(&data).unwrap();
                    assert_eq!(v.len(), 1);
                }
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_list_db_contents() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_list_db";
        let db_pack_info = DBPacketInfo::new(db_name);
        let db_location = DBLocation::new("location1");
        let db_data = DBData::new("this is data".to_string());

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(create_response.unwrap(), SuccessNoData);

        let list_db_contents_invalid_perms1 =
            db_list.list_db_contents(&db_pack_info, &"not a valid key most likely".to_string());
        assert_eq!(
            list_db_contents_invalid_perms1.unwrap_err(),
            InvalidPermissions
        );
        let list_db_contents_invalid_perms2 =
            db_list.list_db_contents(&db_pack_info, &TEST_USER_KEY.to_string());
        match list_db_contents_invalid_perms2.unwrap() {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => match serde_json::from_str::<HashMap<String, String>>(&data) {
                Ok(thing) => {
                    assert_eq!(thing.len(), 0);
                }
                Err(err) => {
                    panic!("{:?}", err);
                }
            },
        }
        let list_db_contents_valid_perms =
            db_list.list_db_contents(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
        match list_db_contents_valid_perms.unwrap() {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => match serde_json::from_str::<HashMap<String, String>>(&data) {
                Ok(thing) => {
                    assert_eq!(thing.len(), 0);
                }
                Err(err) => {
                    panic!("{:?}", err);
                }
            },
        }

        let write_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            &db_data.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(write_response.unwrap(), SuccessNoData);
        let list_db_contents_valid_perms =
            db_list.list_db_contents(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
        match list_db_contents_valid_perms.unwrap() {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => match serde_json::from_str::<HashMap<String, String>>(&data) {
                Ok(thing) => {
                    assert_eq!(thing.len(), 1);
                }
                Err(err) => {
                    panic!("{:?}", err);
                }
            },
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_get_and_set_db_settings() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_add_remove_admin";
        let db_pack_info = DBPacketInfo::new(db_name);
        let new_admin_key = "new admin key that gets added".to_string();
        let new_db_settings = DBSettings::new(
            Duration::from_secs(28),
            (false, true, false),
            (false, false, true),
            vec![new_admin_key],
            vec![],
        );
        assert_ne!(new_db_settings, get_db_test_settings());
        {
            let create_response = db_list.create_db(
                db_name,
                get_db_test_settings(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );

            assert_eq!(create_response.unwrap(), SuccessNoData);
        }

        {
            let missing_perms_get_db_settings1 =
                db_list.get_db_settings(&db_pack_info, &TEST_USER_KEY.to_string());
            assert_eq!(
                missing_perms_get_db_settings1.unwrap_err(),
                InvalidPermissions
            );
            let missing_perms_get_db_settings2 =
                db_list.get_db_settings(&db_pack_info, &"not a working key".to_string());
            assert_eq!(
                missing_perms_get_db_settings2.unwrap_err(),
                InvalidPermissions
            );
            let original_db_settings =
                db_list.get_db_settings(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
            match original_db_settings.unwrap() {
                SuccessNoData => {
                    unreachable!()
                }
                SuccessReply(data) => {
                    let received_original_db_settings: DBSettings =
                        serde_json::from_str(&data).unwrap();
                    assert_eq!(received_original_db_settings, get_db_test_settings());
                }
            }
        }

        {
            let missing_perms_set_db_settings1 = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &TEST_USER_KEY.to_string(),
            );
            assert_eq!(
                missing_perms_set_db_settings1.unwrap_err(),
                InvalidPermissions
            );
            let missing_perms_set_db_settings2 = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &"also not a working key".to_string(),
            );
            assert_eq!(
                missing_perms_set_db_settings2.unwrap_err(),
                InvalidPermissions
            );
            let change_db_settings_response = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(change_db_settings_response.unwrap(), SuccessNoData);
        }
        {
            let missing_perms_get_db_settings1 =
                db_list.get_db_settings(&db_pack_info, &TEST_USER_KEY.to_string());
            assert_eq!(
                missing_perms_get_db_settings1.unwrap_err(),
                InvalidPermissions
            );
            let missing_perms_get_db_settings2 =
                db_list.get_db_settings(&db_pack_info, &"not a working key".to_string());
            assert_eq!(
                missing_perms_get_db_settings2.unwrap_err(),
                InvalidPermissions
            );
            let original_db_settings =
                db_list.get_db_settings(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());

            match original_db_settings.unwrap() {
                SuccessNoData => {
                    unreachable!()
                }
                SuccessReply(data) => {
                    let received_original_db_settings: DBSettings =
                        serde_json::from_str(&data).unwrap();
                    assert_eq!(received_original_db_settings, new_db_settings.clone());
                }
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_get_role() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_get_role";
        let db_pack_info = DBPacketInfo::new(db_name);
        let new_admin_key = "new admin key that gets added".to_string();
        let user_key = "this is a working user key".to_string();
        let new_db_settings = DBSettings::new(
            Duration::from_secs(28),
            (false, true, false),
            (false, false, true),
            vec![new_admin_key.clone()],
            vec![user_key.clone()],
        );

        let create_resp =
            db_list.create_db(db_name, new_db_settings, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(create_resp.unwrap(), SuccessNoData);

        {
            let role = db_list.get_role(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
            match role.unwrap() {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => match serde_json::from_str::<Role>(&role_ser) {
                    Ok(role_deser) => {
                        assert_eq!(role_deser, SuperAdmin)
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &new_admin_key);
            match role.unwrap() {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => match serde_json::from_str::<Role>(&role_ser) {
                    Ok(role_deser) => {
                        assert_eq!(role_deser, Admin)
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &user_key);
            match role.unwrap() {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => match serde_json::from_str::<Role>(&role_ser) {
                    Ok(role_deser) => {
                        assert_eq!(role_deser, User)
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &"not a key at all!!?!".to_string());
            match role.unwrap() {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => match serde_json::from_str::<Role>(&role_ser) {
                    Ok(role_deser) => {
                        assert_eq!(role_deser, Other)
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response.unwrap(), SuccessNoData);
    }

    #[test]
    fn test_delete_data() {
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_delete_data";
        let db_pack_info = DBPacketInfo::new(db_name);
        let db_location = DBLocation::new("location1");
        let db_data = DBData::new("this is data".to_string());

        {
            let create_resp = db_list.create_db(
                db_name,
                get_db_test_settings(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(create_resp.unwrap(), SuccessNoData);
        }

        {
            let write_resp = db_list.write_db(
                &db_pack_info,
                &db_location,
                &db_data.clone(),
                &"not a working key probably".to_string(),
            );
            assert_eq!(write_resp.unwrap_err(), InvalidPermissions);
        }

        {
            let write_resp = db_list.write_db(
                &db_pack_info,
                &db_location,
                &db_data.clone(),
                &TEST_USER_KEY.to_string(),
            );
            assert_eq!(write_resp.unwrap(), SuccessNoData);
        }

        {
            let write_resp = db_list.write_db(
                &db_pack_info,
                &db_location,
                &db_data.clone(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(
                write_resp.unwrap(),
                SuccessReply(db_data.get_data().to_string())
            );
        }

        {
            let get_data_resp = db_list.read_db(
                &db_pack_info,
                &db_location,
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(
                get_data_resp.unwrap(),
                SuccessReply(db_data.get_data().to_string())
            );
        }

        {
            let delete_response = db_list.delete_data(
                &db_pack_info,
                &db_location,
                &"not a working key probably".to_string(),
            );
            assert_eq!(delete_response.unwrap_err(), InvalidPermissions);
        }

        {
            let delete_response =
                db_list.delete_data(&db_pack_info, &db_location, &TEST_USER_KEY.to_string());
            assert_eq!(
                delete_response.unwrap(),
                SuccessReply(db_data.get_data().to_string())
            );
        }

        {
            let delete_response = db_list.delete_data(
                &db_pack_info,
                &db_location,
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(delete_response.unwrap_err(), ValueNotFound);
        }

        {
            let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
            assert_eq!(delete_response.unwrap(), SuccessNoData);
        }
    }
}
