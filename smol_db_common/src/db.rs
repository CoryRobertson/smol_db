//! Contains the struct that represents specific databases.
use crate::db::Role::{Admin, Other, SuperAdmin, User};
use crate::db_content::DBContent;
use crate::db_packets::db_settings::DBSettings;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use crate::statistics::{DBStatistics};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
/// A struct that represents a specific database, with content, and a recent access time.
/// This struct is meant to be called into existence when ever a database is un-cached, and needs to be cached.
pub struct DB {
    db_content: DBContent,
    last_access_time: SystemTime,
    db_settings: DBSettings,
    #[serde(default)]
    statistics: DBStatistics,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Copy, Eq)]
/// Represents the role a user has in a db, given a key.
pub enum Role {
    SuperAdmin,
    Admin,
    User,
    Other,
}

impl Default for DB {
    fn default() -> Self {
        Self {
            db_content: DBContent::default(),
            last_access_time: SystemTime::now(),
            db_settings: DBSettings::default(),
            statistics: DBStatistics::default(),
        }
    }
}

impl DB {

    pub fn new_from_settings(db_settings: DBSettings) -> Self {
        Self {
            db_settings,
            ..Default::default()
        }
    }

    pub fn get_settings(&self) -> &DBSettings {
        &self.db_settings
    }

    pub fn get_settings_mut(&mut self) -> &mut DBSettings {
        &mut self.db_settings
    }

    pub fn set_settings(&mut self, new_settings: DBSettings) {
        self.db_settings = new_settings;
    }

    pub fn get_content_mut(&mut self) -> &mut DBContent {
        &mut self.db_content
    }

    pub fn get_content(&self) -> &DBContent {
        &self.db_content
    }

    pub fn update_access_time(&mut self) {
        self.statistics.add_new_time(self.last_access_time);
        self.last_access_time = SystemTime::now();
    }

    pub fn get_access_time(&self) -> SystemTime {
        self.last_access_time
    }

    /// Returns the given role the client key falls in.
    pub fn get_role(&self, client_key: &String, super_admin_list: &[String]) -> Role {
        if super_admin_list.contains(client_key) {
            SuperAdmin
        } else if self.db_settings.is_admin(client_key) {
            Admin
        } else if self.db_settings.is_user(client_key) {
            User
        } else {
            Other
        }
    }

    /// Returns true if the given key has list permissions
    /// Checks which role the user might fit into depending on `DBSettings`
    pub fn has_list_permissions(&self, client_key: &String, super_admin_list: &[String]) -> bool {
        match self.get_role(client_key, super_admin_list) {
            Admin | SuperAdmin => true,
            User => self.db_settings.get_user_rwx().2,
            Other => self.db_settings.get_other_rwx().2,
        }
    }

    /// Returns true if the given key has read permissions
    /// Checks which role the user might fit into depending on `DBSettings`
    pub fn has_read_permissions(&self, client_key: &String, super_admin_list: &[String]) -> bool {
        match self.get_role(client_key, super_admin_list) {
            Admin | SuperAdmin => true,
            User => self.db_settings.get_user_rwx().0,
            Other => self.db_settings.get_other_rwx().0,
        }
    }

    /// Returns true if the given key has write permissions
    /// Checks which role the user might fit into depending on `DBSettings`
    pub fn has_write_permissions(&self, client_key: &String, super_admin_list: &[String]) -> bool {
        match self.get_role(client_key, super_admin_list) {
            Admin | SuperAdmin => true,
            User => self.db_settings.get_user_rwx().1,
            Other => self.db_settings.get_other_rwx().1,
        }
    }
}
