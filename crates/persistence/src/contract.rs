// Contract tests shared with implementations in other crates:
// compiled outside `cfg(test)` behind the `contract-tests` feature, hence allowing
// `unwrap`, normally reserved for tests.
#![allow(clippy::unwrap_used)]

use domain::Db;
use pollster::block_on;

use crate::codec::DbCodec;
use crate::storage::Storage;

pub fn check_storage_contract<S: Storage>(new_empty_storage: impl Fn() -> S) {
    let storage = new_empty_storage();
    assert_eq!(
        block_on(storage.read()),
        Ok(None),
        "an empty storage returns Ok(None)"
    );

    let every_byte_value: Vec<u8> = (0..=u8::MAX).collect();
    for bytes in [Vec::new(), b"{}".to_vec(), every_byte_value] {
        let storage = new_empty_storage();
        assert_eq!(block_on(storage.write(&bytes)), Ok(()));
        assert_eq!(
            block_on(storage.read()),
            Ok(Some(bytes)),
            "written bytes are read back unchanged"
        );
    }

    let storage = new_empty_storage();
    block_on(storage.write(b"first")).unwrap();
    block_on(storage.write(b"second")).unwrap();
    assert_eq!(
        block_on(storage.read()),
        Ok(Some(b"second".to_vec())),
        "a write replaces the previous one"
    );
}

pub fn check_codec_contract<C: DbCodec>(codec: &C, sample: &Db) {
    for db in [&Db::default(), sample] {
        let bytes = codec.encode(db).unwrap();
        assert_eq!(
            codec.decode(&bytes).as_ref(),
            Ok(db),
            "decode(encode(db)) == db"
        );
    }
    assert!(codec.decode(b"").is_err(), "empty bytes are rejected");
    assert!(
        codec.decode(&[0xff, 0x00, 0x13]).is_err(),
        "arbitrary bytes are rejected"
    );
}
