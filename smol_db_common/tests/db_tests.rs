#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use smol_db_common::db::Role::{Admin, Other, User};
    use smol_db_common::db::DB;
    use smol_db_common::db_packets::db_settings::DBSettings;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_read_permissions() {
        let admin_key = "test_admin_123".to_string();
        let user_key = "test_user_123".to_string();
        let other_key = "".to_string();
        let db1 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (false, false, false),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db2 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, false, false),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db3 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, false, false),
                (false, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };

        assert_eq!(db1.has_read_permissions(&other_key), false);
        assert_eq!(db2.has_read_permissions(&other_key), true);
        assert_eq!(db3.has_read_permissions(&other_key), true);

        assert_eq!(db1.has_read_permissions(&user_key), true);
        assert_eq!(db2.has_read_permissions(&user_key), true);
        assert_eq!(db3.has_read_permissions(&user_key), false);

        assert_eq!(db1.has_read_permissions(&admin_key), true);
        assert_eq!(db2.has_read_permissions(&admin_key), true);
        assert_eq!(db3.has_read_permissions(&admin_key), true);
    }

    #[test]
    fn test_write_permissions() {
        let admin_key = "test_admin_123".to_string();
        let user_key = "test_user_123".to_string();
        let other_key = "".to_string();
        let db1 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (false, false, false),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db2 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, true, false),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db3 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, false, false),
                (true, false, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        assert_eq!(db1.has_write_permissions(&other_key), false);
        assert_eq!(db2.has_write_permissions(&other_key), true);
        assert_eq!(db3.has_write_permissions(&other_key), false);

        assert_eq!(db1.has_write_permissions(&user_key), true);
        assert_eq!(db2.has_write_permissions(&user_key), true);
        assert_eq!(db3.has_write_permissions(&user_key), false);

        assert_eq!(db1.has_write_permissions(&admin_key), true);
        assert_eq!(db2.has_write_permissions(&admin_key), true);
        assert_eq!(db3.has_write_permissions(&admin_key), true);
    }

    #[test]
    fn test_list_permissions() {
        let admin_key = "test_admin_123".to_string();
        let user_key = "test_user_123".to_string();
        let other_key = "".to_string();
        let db1 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (false, false, true),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db2 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, true, false),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        let db3 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (true, false, true),
                (true, false, false),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };
        assert_eq!(db1.has_list_permissions(&other_key), true);
        assert_eq!(db2.has_list_permissions(&other_key), false);
        assert_eq!(db3.has_list_permissions(&other_key), true);

        assert_eq!(db1.has_list_permissions(&user_key), true);
        assert_eq!(db2.has_list_permissions(&user_key), true);
        assert_eq!(db3.has_list_permissions(&user_key), false);

        assert_eq!(db1.has_list_permissions(&admin_key), true);
        assert_eq!(db2.has_list_permissions(&admin_key), true);
        assert_eq!(db3.has_list_permissions(&admin_key), true);
    }

    #[test]
    fn test_get_role() {
        let admin_key = "test_admin_123".to_string();
        let user_key = "test_user_123".to_string();
        let other_key = "".to_string();
        let db1 = DB {
            db_content: Default::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::new(
                Duration::from_secs(30),
                (false, false, true),
                (true, true, true),
                vec![admin_key.clone()],
                vec![user_key.clone()],
            ),
        };

        assert_eq!(db1.get_role(&admin_key), Admin);
        assert_eq!(db1.get_role(&user_key), User);
        assert_eq!(db1.get_role(&other_key), Other);
    }
}
