use crate::group_logs::GroupLog;
use std::io;
use std::u64;

pub(crate) struct KV {
    meta_log: GroupLog,
    kv_log: GroupLog,
}

impl KV {
    #[inline]
    pub(crate) fn new(meta_dir: &'static str, kv_dir: &'static str, limit_per_file: u64) -> Self {
        KV {
            meta_log: GroupLog::new(meta_dir, limit_per_file),
            kv_log: GroupLog::new(kv_dir, limit_per_file),
        }
    }

    // (u8, u64, u64) = (kv_log_index, value_offset, value_length)
    #[inline]
    pub(crate) fn write(
        &mut self,
        key: &[u8],
        value: &[u8],
        fsync: bool,
    ) -> io::Result<(u8, u64, u64)> {
        let kv_data = [*key, *value].concat().as_slice();
        let kv_pos = self.kv_log.write_data(kv_data, fsync)?;
        let dividing_point = kv_pos.1 + key.len() as u64;

        let kv_offset_u8: [u8; 8] = kv_pos.1.to_be_bytes;
        let kv_len_u8: [u8; 8] = kv_pos.2.to_be_bytes;
        let dividing_point_u8: [u8; 8] = dividing_point.to_be_bytes;
        let metadata = [kv_pos.0, kv_offset_u8, kv_len_u8, dividing_point_u8]
            .concat()
            .as_slice();
        self.meta_log.write_data(metadata, fsync)?;
        Ok((
            kv_pos.0,
            kv_pos.1 + dividing_point,
            kv_pos.2 - dividing_point,
        ))
    }

    #[inline]
    pub(crate) fn read_meta(&self) -> io::Result<[u8]> {
        self.meta_log.read_all()
    }

    #[inline]
    pub(crate) fn read_kv(&self, fidx: u8, offset: u64, len: usize) -> io::Result<[u8]> {
        self.kv_log.read_data(fidx, offset, len)
    }
}
