#[macro_use]
extern crate bencher;

use bencher::{black_box, Bencher};
use near_primitives::borsh::maybestd::sync::Arc;
use near_primitives::errors::StorageError;
use skw_vm_store::db::DBCol::ColBlockMerkleTree;
use skw_vm_store::{create_store, DBCol, Store};
use std::time::{Duration, Instant};

/// Run a benchmark to generate `num_keys` keys, each of size `key_size`, then write then
/// in random order to column `col` in store, and then read keys back from `col` in random order.
/// Works only for column configured without reference counting, that is `.is_rc() == false`.
fn benchmark_write_then_read_successful(
    bench: &mut Bencher,
    num_keys: usize,
    key_size: usize,
    max_value_size: usize,
    col: DBCol,
) {
    let store = create_store_in_random_folder();
    let keys = generate_keys(num_keys, key_size);
    write_to_db(&store, &keys, max_value_size, col);

    bench.iter(move || {
        let start = Instant::now();

        let read_records = read_from_db(&store, &keys, col);
        let took = start.elapsed();
        println!(
            "took on avg {:?} op per sec {} got {}/{}",
            took / (num_keys as u32),
            (num_keys as u128) * Duration::from_secs(1).as_nanos() / took.as_nanos(),
            read_records,
            keys.len()
        );
    });
}

/// Create `Store` in a random folder.
fn create_store_in_random_folder() -> Arc<Store> {
    let tmp_dir = tempfile::Builder::new().prefix("_test_clear_column").tempdir().unwrap();
    let store = create_store(tmp_dir.path());
    store
}

/// Generate `count` keys of `key_size` length.
fn generate_keys(count: usize, key_size: usize) -> Vec<Vec<u8>> {
    let mut res: Vec<Vec<u8>> = Vec::new();
    for _k in 0..count {
        let key: Vec<u8> = (0..key_size).map(|_| rand::random::<u8>()).collect();

        res.push(key)
    }
    res
}

/// Read from DB value for given `kyes` in random order for `col`.
/// Works only for column configured without reference counting, that is `.is_rc() == false`.
fn read_from_db(store: &Arc<Store>, keys: &Vec<Vec<u8>>, col: DBCol) -> usize {
    let mut read = 0;
    for _k in 0..keys.len() {
        let r = rand::random::<u32>() % (keys.len() as u32);
        let key = &keys[r as usize];

        let val = store.get(col, key.as_ref()).map_err(|_| StorageError::StorageInternalError);

        if let Ok(Some(x)) = val {
            black_box(x);
            read += 1;
        }
    }
    read
}

/// Write random value of size between `0` and `max_value_size` to given `keys` at specific column
/// `col.`
/// Works only for column configured without reference counting, that is `.is_rc() == false`.
fn write_to_db(store: &Arc<Store>, keys: &[Vec<u8>], max_value_size: usize, col: DBCol) {
    let mut store_update = store.store_update();
    for key in keys.iter() {
        let x: usize = rand::random::<usize>() % max_value_size;
        let val: Vec<u8> = (0..x).map(|_| rand::random::<u8>()).collect();
        // NOTE:  this
        store_update.set(col, key.as_slice().clone(), &val);
    }
    store_update.commit().unwrap();
}

fn benchmark_write_then_read_successful_10m(bench: &mut Bencher) {
    // By adding logs, I've seen a lot of write to keys with size 40, an values with sizes
    // between 10 .. 333.
    // NOTE: ColBlockMerkleTree was chosen to be a column, where `.is_rc() == false`.
    // benchmark_write_then_read_successful(bench, 10_000_000, 40, 333, ColBlockMerkleTree);
    benchmark_write_then_read_successful(bench, 10_000, 40, 333, ColBlockMerkleTree);
}

benchmark_group!(benches, benchmark_write_then_read_successful_10m);

benchmark_main!(benches);