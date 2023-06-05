//! Contains the struct that represents specific databases.
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

    // TODO: a vec of access key hashes here should contain a list of admins for this given db

    // TODO: a vec of access key hashes here should contain a list of users for this given db

    // TODO: add a permissions struct with the following information:
    //  canOthersRead
    //  canOthersWrite
    //  canOthersList
    //  canUsersRead
    //  canUsersWrite
    //  canUsersList
    //  Admins can always read, write, list, and delete.
}

// TODO enum PermissionState that has the following states: isAdmin, isUser: isOther
//  impl block that checks the db vecs for if the users hash is in the user hash list, or the admin hash list, or neither.

// TODO: permissions struct should be a function implementation on DB that checks if a given user can do a given action.
//  e.g. to write_db, after checking cache, or after loading to cache, the db uses the PermissionState enum impl to determine what type of user the input is comming from.
//  after determining the enum, we can bar specific actions from that user.

