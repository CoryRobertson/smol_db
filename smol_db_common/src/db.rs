//! Contains the struct that represents specific databases.
use crate::db::Role::{Admin, Other, SuperAdmin, User};
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
    SuperAdmin,
    Admin,
    User,
    Other,
}

impl DB {
    /// Returns the given role the client key falls in.
    pub fn get_role(&self, client_key: &String, super_admin_list: &Vec<String>) -> Role {
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
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_list_permissions(&self, client_key: &String, super_admin_list: &Vec<String>) -> bool {
        match self.get_role(client_key,super_admin_list) {
            Admin => true,
            User => self.db_settings.get_user_rwx().2,
            Other => self.db_settings.get_other_rwx().2,
            SuperAdmin => { true }
        }
    }

    /// Returns true if the given key has read permissions
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_read_permissions(&self, client_key: &String, super_admin_list: &Vec<String>) -> bool {
        match self.get_role(client_key,super_admin_list) {
            Admin => true,
            User => self.db_settings.get_user_rwx().0,
            Other => self.db_settings.get_other_rwx().0,
            SuperAdmin => { true }
        }
    }

    /// Returns true if the given key has write permissions
    /// Checks which role the user might fit into depending on DBSettings
    pub fn has_write_permissions(&self, client_key: &String, super_admin_list: &Vec<String>) -> bool {
        match self.get_role(client_key,super_admin_list) {
            Admin => true,
            User => self.db_settings.get_user_rwx().1,
            Other => self.db_settings.get_other_rwx().1,
            SuperAdmin => { true }
        }
    }
}
