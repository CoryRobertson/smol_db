//! Module containing a `DBSettings` struct, a struct that represents the various settings a database has.
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
/// Struct describing settings used when creating a db.
pub struct DBSettings {
    /// The duration to wait before removing the given db from the cache.
    pub invalidation_time: Duration,
    /// Triple of the permissions others have to (read,write,list)
    pub can_others_rwx: (bool, bool, bool),
    /// Triple of the permissions users have to (read,write,list)
    pub can_users_rwx: (bool, bool, bool),
    /// Admin list of hashes
    pub admins: Vec<String>,
    /// User list of hashes
    pub users: Vec<String>,
}

impl DBSettings {
    /// Returns a new `DBSettings` given a duration
    pub fn new(
        invalidation_time: Duration,
        can_others_rwx: (bool, bool, bool),
        can_users_rwx: (bool, bool, bool),
        admins: Vec<String>,
        users: Vec<String>,
    ) -> Self {
        Self {
            invalidation_time,
            can_others_rwx,
            can_users_rwx,
            admins,
            users,
        }
    }

    /// Get a list of the keys who are marked as admins of this database, admins have permission to change any piece of data in the database, and view all of it.
    pub fn get_admin_list(&self) -> &Vec<String> {
        &self.admins
    }

    /// Get a list of the keys who are marked as users of this database, users commonly have slightly elevated privileges compared to non-users (others).
    pub fn get_user_list(&self) -> &Vec<String> {
        &self.users
    }

    /// Adds an admin to the DB
    pub fn add_admin(&mut self, hash: String) {
        self.admins.push(hash);
    }

    /// Adds a user to a DB
    pub fn add_user(&mut self, hash: String) {
        self.users.push(hash);
    }

    /// Removes a user from the db settings
    /// Returns true if it found the users hash, false if the users hash was not found
    pub fn remove_user(&mut self, hash: &str) -> bool {
        let it = self.users.clone();
        let mut removed = false;
        for (index, item) in it.iter().enumerate() {
            if hash == item {
                self.users.remove(index);
                removed = true;
            }
        }
        removed
    }

    /// Removes an admin from the db settings
    /// Returns true if the given admin was removed, false if not.
    pub fn remove_admin(&mut self, hash: &str) -> bool {
        let it = self.admins.clone();
        let mut removed = false;
        for (index, item) in it.iter().enumerate() {
            if hash == item {
                self.admins.remove(index);
                removed = true;
            }
        }
        removed
    }

    /// Returns true if the given key is an admin key
    pub fn is_admin(&self, client_key: &String) -> bool {
        self.admins.contains(client_key)
    }

    /// Returns true if the given key is a user key
    pub fn is_user(&self, client_key: &String) -> bool {
        self.users.contains(client_key)
    }

    /// Returns the permissions of the database regarding the users
    pub fn get_user_rwx(&self) -> (bool, bool, bool) {
        self.can_users_rwx
    }

    /// Returns the permissions of the database regarding the others
    pub fn get_other_rwx(&self) -> (bool, bool, bool) {
        self.can_others_rwx
    }

    /// Returns the invalidation time duration
    pub fn get_invalidation_time(&self) -> Duration {
        self.invalidation_time
    }
}

impl Default for DBSettings {
    fn default() -> Self {
        Self {
            invalidation_time: Duration::from_secs(30),
            can_others_rwx: (false, false, false),
            can_users_rwx: (true, true, true),
            admins: vec![],
            users: vec![],
        }
    }
}
