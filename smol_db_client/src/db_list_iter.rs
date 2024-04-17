use crate::prelude::SmolDbClient;
use smol_db_common::prelude::DBPacket;
use smol_db_common::{prelude::DBPacketResponseError, prelude::DBSuccessResponse};
use std::io::{Read, Write};
use tracing::{debug, info};

/// `TableIter` stops the stream to the DB when it is dropped or runs out of values in the DB automatically
pub struct DBListIter<'a>(pub(crate) &'a mut SmolDbClient);

impl Drop for DBListIter<'_> {
    fn drop(&mut self) {
        debug!("DB list iter dropped");
        // let _ = self.0.get_socket().set_read_timeout(None);
        let _ = self.0.send_packet(&DBPacket::EndStreamRead); // attempt to end the read stream when the table iter is dropped
    }
}

impl Iterator for DBListIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: [u8; 1024] = [0; 1024];

        // I have not had unit testing issues with tests relating to this iterator, but this solves it for talbe iter, though taking an inbetween packet solves it as well for now
        // self.0.get_socket().set_read_timeout(Some(Duration::from_secs(5))).ok()?;

        let request_new_packet = serde_json::to_string(&DBPacket::ReadyForNextItem).unwrap();

        let _ = self
            .0
            .get_socket()
            .write(request_new_packet.as_bytes())
            .ok()?;

        debug!("Reading from sockets");

        let read_len1 = self.0.get_socket().read(&mut buf).ok()?;

        let key = String::from_utf8(buf[0..read_len1].to_vec()).unwrap();

        if serde_json::from_str::<Result<DBSuccessResponse<String>, DBPacketResponseError>>(
            &key[0..read_len1],
        )
        .is_ok()
        {
            info!("DB list iter returned none in key read");
            return None;
        }

        debug!("{:?}", key);

        Some(key)
    }
}
