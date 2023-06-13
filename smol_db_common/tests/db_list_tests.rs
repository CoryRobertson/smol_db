#[cfg(test)]
#[allow(unused_imports)]
mod tests {

    use smol_db_common::db::Role;
    use smol_db_common::db::Role::{Admin, Other, SuperAdmin, User};
    use smol_db_common::db_data::DBData;
    use smol_db_common::db_list::DBList;
    use smol_db_common::db_packets::db_location::DBLocation;
    use smol_db_common::db_packets::db_packet_info::DBPacketInfo;
    use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
    use smol_db_common::db_packets::db_packet_response::DBPacketResponse::{Error, SuccessNoData, SuccessReply};
    use smol_db_common::db_packets::db_packet_response::DBPacketResponseError::{
        DBAlreadyExists, DBNotFound, InvalidPermissions, UserNotFound,
    };
    use smol_db_common::db_packets::db_settings::DBSettings;
    use std::collections::HashMap;
    use std::fs::File;
    use std::hash::Hash;
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
        let db_list = get_db_list_for_testing();
        db_list
            .super_admin_hash_list
            .write()
            .unwrap()
            .push(TEST_SUPER_ADMIN_KEY.to_string());
        let db_name = "test_dblist_1_create";
        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );

        let _ = create_response.as_result().expect("Create response failed");

        let create_response_db_already_exists = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(
            create_response_db_already_exists,
            Error(DBAlreadyExists)
        );

        let create_response_db_invalid_perms = db_list.create_db(
            "other_db",
            get_db_test_settings(),
            &"this is not an admin key".to_string(),
        );

        assert_eq!(
            create_response_db_invalid_perms,
            Error(InvalidPermissions)
        );

        // clean up unit test files
        fs::remove_file("test_dblist_1_create").unwrap();
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

        let _ = create_response.as_result().expect("Create response failed");

        let invalid_perms_delete_response =
            db_list.delete_db(db_name, &"not a working admin key".to_string());
        assert_eq!(
            invalid_perms_delete_response,
            Error(InvalidPermissions)
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);

        match File::open(db_name) {
            Ok(f) => {
                panic!("db not deleted {:?}", f)
            }
            Err(_) => {}
        }

        let delete_response_not_listed =
            db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(
            delete_response_not_listed,
            Error(DBNotFound)
        );
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

        let _ = create_response.as_result().expect("Create response failed");

        let write_invalid_perms = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &"not a working client key".to_string(),
        );
        assert_eq!(
            write_invalid_perms,
            Error(InvalidPermissions)
        );

        let write_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(write_response, SuccessNoData);

        let write_response2 = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(
            write_response2,
            SuccessReply(db_data.get_data().to_string())
        );

        let read_response = db_list.read_db(
            &db_pack_info,
            &db_location,
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(
            read_response,
            SuccessReply(db_data.get_data().to_string())
        );

        let read_user_perms_response =
            db_list.read_db(&db_pack_info, &db_location, &TEST_USER_KEY.to_string());
        assert_eq!(
            read_user_perms_response,
            SuccessReply(db_data.get_data().to_string())
        );

        let read_invalid_perms_response = db_list.read_db(
            &db_pack_info,
            &db_location,
            &"not a user key or an admin key".to_string(),
        );
        assert_eq!(
            read_invalid_perms_response,
            Error(InvalidPermissions)
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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

        let _ = create_response.as_result().expect("Create response failed");

        // add user without perms, and with perms, and the test users key
        let add_user_invalid_perms1 = db_list.add_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_USER_KEY.to_string(),
        );
        assert_eq!(
            add_user_invalid_perms1,
            Error(InvalidPermissions)
        );
        let add_user_invalid_perms2 = db_list.add_user(
            &db_pack_info,
            new_user_key.clone(),
            &"not a working key".to_string(),
        );
        assert_eq!(
            add_user_invalid_perms2,
            Error(InvalidPermissions)
        );
        let add_user_response = db_list.add_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(add_user_response, SuccessNoData);

        // try writing data to the db with the perms of the new user
        let write_with_new_user_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &new_user_key.to_string(),
        );
        assert_eq!(
            write_with_new_user_response,
            SuccessNoData
        );
        let read_with_new_user_response =
            db_list.read_db(&db_pack_info, &db_location, &new_user_key.to_string());
        assert_eq!(
            read_with_new_user_response,
            SuccessReply(db_data.clone().get_data().to_string())
        );

        // remove user with invalid perms, then eventually remove the user with an admin perm, and try removing the user again and note that the user is not found
        let remove_user_invalid_perms1 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_USER_KEY.to_string(),
        );
        assert_eq!(
            remove_user_invalid_perms1,
            Error(InvalidPermissions)
        );
        let remove_user_invalid_perms2 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone(),
            &"not a working key".to_string(),
        );
        assert_eq!(
            remove_user_invalid_perms2,
            Error(InvalidPermissions)
        );
        let remove_user_response1 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(remove_user_response1, SuccessNoData);
        let remove_user_response2 = db_list.remove_user(
            &db_pack_info,
            new_user_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(remove_user_response2, Error(UserNotFound));

        // write to the db with invalid perms of the added user, who was removed, also attempt to read using the removed users key
        let write_with_new_user_response2 = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &new_user_key.to_string(),
        );
        assert_eq!(
            write_with_new_user_response2,
            Error(InvalidPermissions)
        );
        let read_with_new_user_response2 =
            db_list.read_db(&db_pack_info, &db_location, &new_user_key.to_string());
        assert_eq!(
            read_with_new_user_response2,
            Error(InvalidPermissions)
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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

        let _ = create_response.as_result().expect("Create response failed");

        let add_admin_without_perms1 = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &"this is not a working key".to_string(),
        );
        assert_eq!(
            add_admin_without_perms1,
            Error(InvalidPermissions)
        );
        let add_admin_without_perms2 = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &TEST_USER_KEY.to_string(),
        );
        assert_eq!(
            add_admin_without_perms2,
            Error(InvalidPermissions)
        );
        let add_admin_with_perms = db_list.add_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(add_admin_with_perms, SuccessNoData);

        let new_admin_add_user =
            db_list.add_user(&db_pack_info, new_user_key.clone(), &new_admin_key.clone());
        assert_eq!(new_admin_add_user, SuccessNoData);

        let remove_admin_without_perms1 = db_list.remove_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &"this is not a working key".to_string(),
        );
        assert_eq!(
            remove_admin_without_perms1,
            Error(InvalidPermissions)
        );
        let remove_admin_without_perms2 =
            db_list.remove_admin(&db_pack_info, new_admin_key.clone(), &new_admin_key.clone());
        assert_eq!(
            remove_admin_without_perms2,
            Error(InvalidPermissions)
        );
        let remove_admin_success_response = db_list.remove_admin(
            &db_pack_info,
            new_admin_key.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(
            remove_admin_success_response,
            SuccessNoData
        );

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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
            match db_list_response {
                SuccessNoData => {}
                SuccessReply(data) => {
                    let v = serde_json::from_str::<Vec<DBPacketInfo>>(&data).unwrap();
                    assert_eq!(v.len(), 0);
                }
                Error(_) => {}
            }
        }

        let create_response = db_list.create_db(
            db_name,
            get_db_test_settings(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );

        let _ = create_response.as_result().expect("Create response failed");

        {
            let db_list_response = db_list.list_db();
            match db_list_response {
                SuccessNoData => {}
                SuccessReply(data) => {
                    let v = serde_json::from_str::<Vec<DBPacketInfo>>(&data).unwrap();
                    assert_eq!(v.len(), 1);
                }
                Error(_) => {}
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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

        let _ = create_response.as_result().expect("Create response failed");

        let list_db_contents_invalid_perms1 =
            db_list.list_db_contents(&db_pack_info, &"not a valid key most likely".to_string());
        assert_eq!(
            list_db_contents_invalid_perms1,
            Error(InvalidPermissions)
        );
        let list_db_contents_invalid_perms2 =
            db_list.list_db_contents(&db_pack_info, &TEST_USER_KEY.to_string());
        match list_db_contents_invalid_perms2 {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => {
                match serde_json::from_str::<HashMap<String, String>>(&data) {
                    Ok(thing) => {
                        assert_eq!(thing.len(), 0);
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
            Error(err) => {
                panic!("{:?}", err);
            }
        }
        let list_db_contents_valid_perms =
            db_list.list_db_contents(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
        match list_db_contents_valid_perms {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => {
                match serde_json::from_str::<HashMap<String, String>>(&data) {
                    Ok(thing) => {
                        assert_eq!(thing.len(), 0);
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
            Error(err) => {
                panic!("{:?}", err);
            }
        }

        let write_response = db_list.write_db(
            &db_pack_info,
            &db_location,
            db_data.clone(),
            &TEST_SUPER_ADMIN_KEY.to_string(),
        );
        assert_eq!(write_response, SuccessNoData);
        let list_db_contents_valid_perms =
            db_list.list_db_contents(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
        match list_db_contents_valid_perms {
            SuccessNoData => {
                panic!("No data received from db contents? Bad packet possibly?");
            }
            SuccessReply(data) => {
                match serde_json::from_str::<HashMap<String, String>>(&data) {
                    Ok(thing) => {
                        assert_eq!(thing.len(), 1);
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
            Error(err) => {
                panic!("{:?}", err);
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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

            let _ = create_response.as_result().expect("Create response failed");
        }

        {
            let missing_perms_get_db_settings1 =
                db_list.get_db_settings(&db_pack_info, &TEST_USER_KEY.to_string());
            assert_eq!(
                missing_perms_get_db_settings1,
                Error(InvalidPermissions)
            );
            let missing_perms_get_db_settings2 =
                db_list.get_db_settings(&db_pack_info, &"not a working key".to_string());
            assert_eq!(
                missing_perms_get_db_settings2,
                Error(InvalidPermissions)
            );
            let original_db_settings =
                db_list.get_db_settings(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
            let received_original_db_settings: DBSettings =
                serde_json::from_str(original_db_settings.as_result().unwrap().unwrap()).unwrap();
            assert_eq!(received_original_db_settings, get_db_test_settings());
        }

        {
            let missing_perms_set_db_settings1 = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &TEST_USER_KEY.to_string(),
            );
            assert_eq!(
                missing_perms_set_db_settings1,
                Error(InvalidPermissions)
            );
            let missing_perms_set_db_settings2 = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &"also not a working key".to_string(),
            );
            assert_eq!(
                missing_perms_set_db_settings2,
                Error(InvalidPermissions)
            );
            let change_db_settings_response = db_list.change_db_settings(
                &db_pack_info,
                new_db_settings.clone(),
                &TEST_SUPER_ADMIN_KEY.to_string(),
            );
            assert_eq!(change_db_settings_response, SuccessNoData);
        }
        {
            let missing_perms_get_db_settings1 =
                db_list.get_db_settings(&db_pack_info, &TEST_USER_KEY.to_string());
            assert_eq!(
                missing_perms_get_db_settings1,
                Error(InvalidPermissions)
            );
            let missing_perms_get_db_settings2 =
                db_list.get_db_settings(&db_pack_info, &"not a working key".to_string());
            assert_eq!(
                missing_perms_get_db_settings2,
                Error(InvalidPermissions)
            );
            let original_db_settings =
                db_list.get_db_settings(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
            let received_original_db_settings: DBSettings =
                serde_json::from_str(original_db_settings.as_result().unwrap().unwrap()).unwrap();
            assert_eq!(received_original_db_settings, new_db_settings.clone());
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
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
        assert_eq!(create_resp, SuccessNoData);

        {
            let role = db_list.get_role(&db_pack_info, &TEST_SUPER_ADMIN_KEY.to_string());
            match role {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => {
                    match serde_json::from_str::<Role>(&role_ser) {
                        Ok(role_deser) => {
                            assert_eq!(role_deser, SuperAdmin)
                        }
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    }
                }
                Error(err) => {
                    panic!("bad response from get role: {:?}", err)
                }
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &new_admin_key);
            match role {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => {
                    match serde_json::from_str::<Role>(&role_ser) {
                        Ok(role_deser) => {
                            assert_eq!(role_deser, Admin)
                        }
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    }
                }
                Error(err) => {
                    panic!("bad response from get role: {:?}", err)
                }
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &user_key);
            match role {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => {
                    match serde_json::from_str::<Role>(&role_ser) {
                        Ok(role_deser) => {
                            assert_eq!(role_deser, User)
                        }
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    }
                }
                Error(err) => {
                    panic!("bad response from get role: {:?}", err)
                }
            }
        }

        {
            let role = db_list.get_role(&db_pack_info, &"not a key at all!!?!".to_string());
            match role {
                SuccessNoData => {
                    panic!("bad response from get role")
                }
                SuccessReply(role_ser) => {
                    match serde_json::from_str::<Role>(&role_ser) {
                        Ok(role_deser) => {
                            assert_eq!(role_deser, Other)
                        }
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    }
                }
                Error(err) => {
                    panic!("bad response from get role: {:?}", err)
                }
            }
        }

        let delete_response = db_list.delete_db(db_name, &TEST_SUPER_ADMIN_KEY.to_string());
        assert_eq!(delete_response, SuccessNoData);
    }
}
