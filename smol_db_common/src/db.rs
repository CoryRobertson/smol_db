//! Contains the struct that represents specific databases.
use crate::db::Role::{Admin, Other, User};
use crate::db_content::DBContent;
use crate::db_packets::db_settings::DBSettings;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that represents a specific database, with content, and a recent access time.
/// This struct is meant to be called into existence when ever a database is un-cached, and needs to be cached.
pub struct DB {
    pub db_content: DBContent,
    pub last_access_time: SystemTime,
    pub db_settings: DBSettings,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
/// Represents the role a user has in a db, given a key.
pub enum Role {
    Admin,
    User,
    Other,
}

impl DB {
    /// Returns the given role the client key falls in.
    pub fn get_role(&self, client_key: &String) -> Role {
        if self.db_settings.is_admin(client_key) {
            Admin
        } else if self.db_settings.is_user(client_key) {
            User
        } else {
            Other
        }
    }

    /// Returns true if the given key has list permissions
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_list_permissions(&self, client_key: &String) -> bool {
        match self.get_role(client_key) {
            Admin => true,
            User => self.db_settings.get_user_rwx().2,
            Other => self.db_settings.get_other_rwx().2,
        }
    }

    /// Returns true if the given key has read permissions
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_read_permissions(&self, client_key: &String) -> bool {
        match self.get_role(client_key) {
            Admin => true,
            User => self.db_settings.get_user_rwx().0,
            Other => self.db_settings.get_other_rwx().0,
        }
    }

    /// Returns true if the given key has write permissions
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_write_permissions(&self, client_key: &String) -> bool {
        match self.get_role(client_key) {
            Admin => true,
            User => self.db_settings.get_user_rwx().1,
            Other => self.db_settings.get_other_rwx().1,
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::db::Role::{Admin, Other, User};
    use crate::db::DB;
    use crate::db_packets::db_settings::DBSettings;
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
