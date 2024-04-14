use crate::prelude::SmolDbClient;
use smol_db_common::prelude::{DBPacket, DBPacketResponseError, DBSuccessResponse};
use std::io::{Read, Write};
use tracing::{debug, info};

/// `TableIter` stops the stream to the DB when it is dropped or runs out of values in the DB automatically
pub struct TableIter<'a>(pub(crate) &'a mut SmolDbClient);

impl Drop for TableIter<'_> {
    fn drop(&mut self) {
        debug!("Table iter dropped");
        let _ = self.0.send_packet(&DBPacket::EndStreamRead); // attempt to end the read stream when the table iter is dropped
                                                              // we don't care if this fails, it's just nice if it doesn't
    }
}

impl Iterator for TableIter<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: [u8; 1024] = [0; 1024];

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
            info!("Table iter returned none in key read");
            return None;
        }

        let mut buf: [u8; 1024] = [0; 1024];

        let read_len2 = self.0.get_socket().read(&mut buf).ok()?;

        let value = String::from_utf8(buf[0..read_len2].to_vec()).unwrap();
        if serde_json::from_str::<Result<DBSuccessResponse<String>, DBPacketResponseError>>(
            &value[0..read_len2],
        )
        .is_ok()
        {
            info!("Table iter returned none in value read");
            return None;
        }

        debug!("{:?}", (&key, &value));

        Some((key, value))
    }
}
