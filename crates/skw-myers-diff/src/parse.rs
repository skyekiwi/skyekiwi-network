use crate::types::{ConsolidatedDiffOp};

fn pad_size(size: usize) -> [u8; 4] {
    let mut v = [0, 0, 0, 0];
    v[3] = (size & 0xff) as u8;
    v[2] = ((size >> 8) & 0xff) as u8;
    v[1] = ((size >> 16) & 0xff) as u8;
    v[0] = ((size >> 24) & 0xff) as u8;
    v
}

fn unpad_size(size: &[u8; 4]) -> usize {
    if size.len() != 4 {
        panic!("Invalid size");
    }
    return (
        size[3] as usize | 
        ((size[2] as usize) << 8) | 
        ((size[1] as usize) << 16) | 
        ((size[0] as usize) << 24)
    ).into();
}

pub fn diff_ops_to_bytes(ops: Vec<ConsolidatedDiffOp>) -> Vec<u8> {
    let mut v = Vec::new();
    for op in ops {
        match op {
            ConsolidatedDiffOp::Equal(offset, len) => {
                let mut v_op = Vec::new();
                v_op.push(0);
                v_op.extend_from_slice(&pad_size(offset));
                v_op.extend_from_slice(&pad_size(len));
                v.extend_from_slice(&v_op);
            },
            ConsolidatedDiffOp::Insert(ins) => {
                let mut v_op = Vec::new();
                v_op.push(1);
                v_op.extend_from_slice(&pad_size(ins.len()));
                v_op.extend_from_slice(&ins[..]);
                v.extend_from_slice(&v_op);
            },
            _ => {}
            // ConsolidatedDiffOp::Delete(offset, len) => {
            //     let mut v_op = Vec::new();
            //     v_op.push(2);
            //     v_op.extend_from_slice(&pad_size(offset));
            //     v_op.extend_from_slice(&pad_size(len));
            //     v.extend_from_slice(&v_op);
            // },
        }
    }
    v
}

pub fn bytes_to_diff_ops(bytes: &[u8]) -> Vec<ConsolidatedDiffOp> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let op_type = bytes[i];
        i += 1;

        match op_type {
            0 => {
                ops.push(ConsolidatedDiffOp::Equal(
                    unpad_size(&bytes[i..i + 4].try_into().expect("Invalid size")), 
                    unpad_size(&bytes[i + 4..i + 8].try_into().expect("Invalid size"))
                ));
                i += 8;
            },
            1 => {
                let a = unpad_size(&bytes[i..i + 4].try_into().expect("Invalid size"));
                i += 4;
                let v = &bytes[i..i + a];
                ops.push(ConsolidatedDiffOp::Insert(v.to_vec()));
                i += a;
            },
            // 2 => {
            //     ops.push(ConsolidatedDiffOp::Delete(
            //         unpad_size(&bytes[i..i + 4].try_into().expect("Invalid size")), 
            //         unpad_size(&bytes[i + 4..i + 8].try_into().expect("Invalid size"))
            //     ));
            //     i += 8;
            // },
            2 => {}
            _ => {
                panic!("Invalid op type");
            },
        }
    }
    ops
}


#[cfg(test)]
use crate::{diff, patch};

#[test]
fn test_pad_size() {
    assert_eq!(pad_size(0), [0, 0, 0, 0]);
    assert_eq!(pad_size(1), [0, 0, 0, 1]);
    assert_eq!(pad_size(511), [0, 0, 1, 255]);
    assert_eq!(pad_size(65_536), [0, 1, 0, 0]);
    assert_eq!(pad_size(131_071), [0, 1, 255, 255]);
    assert_eq!(pad_size(16_777_216), [1, 0, 0, 0]);
    assert_eq!(pad_size(4_278_190_080), [255, 0, 0, 0]);
}



#[test]
fn test_unpad_size() {
    assert_eq!(unpad_size(&[0, 0, 0, 0]), 0);
    assert_eq!(unpad_size(&[0, 0, 0, 1]), 1);
    assert_eq!(unpad_size(&[0, 0, 1, 255]), 511);
    assert_eq!(unpad_size(&[0, 1, 0, 0]), 65_536);
    assert_eq!(unpad_size(&[0, 1, 255, 255]), 131_071);
    assert_eq!(unpad_size(&[1, 0, 0, 0]), 16_777_216);
    assert_eq!(unpad_size(&[255, 0, 0, 0]), 4_278_190_080);
}

#[test]
fn test_diff_ops_to_bytes() {
    let ops = vec![
        ConsolidatedDiffOp::Equal(0, 1),
        ConsolidatedDiffOp::Insert(vec![1, 2, 3]),
        // ConsolidatedDiffOp::Delete(0, 1),
    ];
    let bytes = diff_ops_to_bytes(ops);
    let ops = bytes_to_diff_ops(&bytes);
    assert_eq!(ops, vec![
        ConsolidatedDiffOp::Equal(0, 1),
        ConsolidatedDiffOp::Insert(vec![1, 2, 3]),
        // ConsolidatedDiffOp::Delete(0, 1),
    ]);
}

#[test]
fn test_e2e() {
    macro_rules! random_bytes{
        ($len:expr) => ({
            let mut bytes = [0_u8; $len];
            for byte in bytes.iter_mut() {
                *byte = rand::random::<u8>();
            }
            bytes
        })
    }

    let mut loops = 1;

    while loops > 0 {
        let mut old = random_bytes!(1000);
        let mut new = random_bytes!(1000);

        let res = diff(&mut old[..], &mut new[..]);

        let bytes = diff_ops_to_bytes(res);

        println!("{:?}", bytes.len());
        let r = bytes_to_diff_ops(&bytes);

        let recovered = patch(r, &old[..]);
        assert_eq!(recovered, new);

        loops -= 1;
    }
}