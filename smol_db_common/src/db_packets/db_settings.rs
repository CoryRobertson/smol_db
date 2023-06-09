use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Struct describing settings used when creating a db.
pub struct DBSettings {
    /// The duration to wait before removing the given db from the cache.
    invalidation_time: Duration,
    /// Triple of the permissions others have to (read,write,list)
    can_others_rwx: (bool,bool,bool),
    /// Triple of the permissions users have to (read,write,list)
    can_users_rwx: (bool,bool,bool),
    /// Admin list of hashes
    admins: Vec<String>,
    /// User list of hashes
    users: Vec<String>,
}

impl DBSettings {
    /// Returns a new DBSettings given a duration
    pub fn new(invalidation_time: Duration,
               can_others_rwx: (bool,bool,bool),
               can_users_rwx: (bool,bool,bool),
               admins: Vec<String>,
               users: Vec<String>
    ) -> Self {
        Self { invalidation_time, can_others_rwx, can_users_rwx, admins, users }
    }

    pub fn add_admin(&mut self, hash: String) {
        self.admins.push(hash);
    }

    pub fn add_user(&mut self, hash: String) {
        self.users.push(hash);
    }

    pub fn is_admin(&self, client_key: &String) -> bool {
        self.admins.contains(client_key)
    }

    pub fn is_user(&self, client_key: &String) -> bool {
        self.users.contains(client_key)
    }

    pub fn get_user_rwx(&self) -> (bool, bool, bool) {
        self.can_users_rwx
    }

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
        Self{
            invalidation_time: Duration::from_secs(30),
            can_others_rwx: (false, false, false),
            can_users_rwx: (true,true,true),
            admins: vec![],
            users: vec![],
        }
    }
}