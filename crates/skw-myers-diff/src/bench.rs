#[macro_use]
extern crate bencher;

use bencher::{Bencher};

use skw_myers_diff::{diff, patch, diff_ops_to_bytes, bytes_to_diff_ops};

macro_rules! random_bytes{
	($len:expr) => ({
		let mut bytes = [0_u8; $len];
		for byte in bytes.iter_mut() {
			*byte = rand::random::<u8>();
		}
		bytes
	})
}

fn diff_n_patch(bench: &mut Bencher) {
    let mut original = random_bytes!(1000);
    let mut modified = random_bytes!(10000);

    bench.iter(|| {
        let res = diff(&mut original[..], &mut modified[..]);
        let r = patch(res, &original[..]);

        assert_eq!(r, modified);
    });
}

fn parse_diff_op(bench: &mut Bencher) {
    let mut original = random_bytes!(1000);
    let mut modified = random_bytes!(100000);
    let res = diff(&mut original[..], &mut modified[..]);

    bench.iter(|| {
        let bytes = diff_ops_to_bytes(res.clone());
        let r = bytes_to_diff_ops(&bytes[..]);
        assert_eq!(r, res);
    });
}

benchmark_group!(benches, parse_diff_op, diff_n_patch);
benchmark_main!(benches);