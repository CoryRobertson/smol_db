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
}
