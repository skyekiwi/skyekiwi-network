use crate::types::{DiffOp, WrappedBytes, ConsolidatedDiffOp, V};

#[allow(dead_code)]
#[derive(Debug)]
struct Snake {
    x_start: usize,
    y_start: usize,
    x_end: usize,
    y_end: usize,
}

fn max_d(len1: usize, len2: usize) -> usize {
    // XXX look into reducing the need to have the additional '+ 1'
    (len1 + len2 + 1) / 2 + 1
}

// The divide part of a divide-and-conquer strategy. A D-path has D+1 snakes some of which may
// be empty. The divide step requires finding the ceil(D/2) + 1 or middle snake of an optimal
// D-path. The idea for doing so is to simultaneously run the basic algorithm in both the
// forward and reverse directions until furthest reaching forward and reverse paths starting at
// opposing corners 'overlap'.
fn find_middle_snake(
    old: WrappedBytes,
    new: WrappedBytes,
    vf: &mut V,
    vb: &mut V,
) -> (isize, Snake) {
    let n = old.len();
    let m = new.len();

    // By Lemma 1 in the paper, the optimal edit script length is odd or even as `delta` is odd
    // or even.
    let delta = n as isize - m as isize;
    let odd = delta & 1 == 1;

    // The initial point at (0, -1)
    vf[1] = 0;
    // The initial point at (N, M+1)
    vb[1] = 0;

    // We only need to explore ceil(D/2) + 1
    let d_max = max_d(n, m);
    assert!(vf.len() >= d_max);
    assert!(vb.len() >= d_max);

    for d in 0..d_max as isize {
        // Forward path
        for k in (-d..=d).rev().step_by(2) {
            let mut x = if k == -d || (k != d && vf[k - 1] < vf[k + 1]) {
                vf[k + 1]
            } else {
                vf[k - 1] + 1
            };
            let mut y = (x as isize - k) as usize;

            // The coordinate of the start of a snake
            let (x0, y0) = (x, y);
            //  While these sequences are identical, keep moving through the graph with no cost
            if let (Some(s1), Some(s2)) = (old.get(x..), new.get(y..)) {
                let advance = common_prefix_len(s1, s2);
                x += advance;
                y += advance;
            }

            // This is the new best x value
            vf[k] = x;
            // Only check for connections from the forward search when N - M is odd
            // and when there is a reciprocal k line coming from the other direction.
            if odd && (k - delta).abs() <= (d - 1) {
                // TODO optimize this so we don't have to compare against n
                if vf[k] + vb[-(k - delta)] >= n {
                    // Return the snake
                    let snake = Snake {
                        x_start: x0,
                        y_start: y0,
                        x_end: x,
                        y_end: y,
                    };
                    // Edit distance to this snake is `2 * d - 1`
                    return (2 * d - 1, snake);
                }
            }
        }

        // Backward path
        for k in (-d..=d).rev().step_by(2) {
            let mut x = if k == -d || (k != d && vb[k - 1] < vb[k + 1]) {
                vb[k + 1]
            } else {
                vb[k - 1] + 1
            };
            let mut y = (x as isize - k) as usize;

            // The coordinate of the start of a snake
            let (x0, y0) = (x, y);
            if x < n && y < m {
                let advance = common_suffix_len(old.slice(..n - x), new.slice(..m - y));
                x += advance;
                y += advance;
            }

            // This is the new best x value
            vb[k] = x;

            if !odd && (k - delta).abs() <= d {
                // TODO optimize this so we don't have to compare against n
                if vb[k] + vf[-(k - delta)] >= n {
                    // Return the snake
                    let snake = Snake {
                        x_start: n - x,
                        y_start: m - y,
                        x_end: n - x0,
                        y_end: m - y0,
                    };
                    // Edit distance to this snake is `2 * d`
                    return (2 * d, snake);
                }
            }
        }

        // TODO: Maybe there's an opportunity to optimize and bail early?
    }

    unreachable!("unable to find a middle snake");
}

fn common_prefix_len(a: WrappedBytes, b: WrappedBytes) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 || b_len == 0 {
        return 0;
    }

    if a.inner_get(0..1) != b.inner_get(0..1) {
        return 0;
    }

    let mut t = 0;
    while t < a_len && t < b_len && a.inner_get(t..t + 1) == b.inner_get(t..t + 1) {
        t += 1;
    }
    t
    // let mut m = 0;
    // let mut ma = std::cmp::min(a.len(), b.len());
    // let mut mid = ma;
    // let mut start = 0;

    // while m < mid {
    //     if a.inner_get(start .. m) == b.inner_get(start .. m) {
    //         m = mid; start = m;
    //     } else {
    //         ma = mid;
    //     }
    //     mid = (ma - m) / 2 + m;
    // }

    // mid
}

fn common_suffix_len(a: WrappedBytes, b: WrappedBytes) -> usize {
    let a_len = a.len();
    let b_len = b.len();


    if a_len == 0 || b_len == 0 {
        return 0;
    }

    if a.inner_get(a_len - 1..a_len) != b.inner_get(b_len - 1..b_len) {
        return 0;
    }

    let mut m = 0;
    let mut ma = std::cmp::min(a_len, b_len);
    let mut mid = ma;
    let mut end = 0;

    while m < mid {
        if a.inner_get(a_len - mid..a_len - end) == b.inner_get(b_len - mid..b_len - end) {
            m = mid; end = m;
        } else {
            ma = mid;
        }
        mid = (ma - m) / 2 + m;
    }

    mid
}

fn conquer<'a>(
    mut old: WrappedBytes<'a>,
    mut new: WrappedBytes<'a>,
    vf: &mut V,
    vb: &mut V,
    solution: &mut Vec<DiffOp<'a>>,
) {
    // Check for common prefix
    let common_prefix_len = common_prefix_len(old, new);
    if common_prefix_len > 0 {

        let common_prefix = DiffOp::Equal(
            old.slice(..common_prefix_len),
            new.slice(..common_prefix_len),
        );
        solution.push(common_prefix);
    }
    old = old.slice(common_prefix_len..old.len());
    new = new.slice(common_prefix_len..new.len());

    // Check for common suffix
    let common_suffix_len = common_suffix_len(old, new);    
    let common_suffix = DiffOp::Equal(
        old.slice(old.len() - common_suffix_len..old.len()),
        new.slice(new.len() - common_suffix_len..new.len()),
    );

    old = old.slice(..old.len() - common_suffix_len);
    new = new.slice(..new.len() - common_suffix_len);

    if old.is_empty() && new.is_empty() {
        // Do nothing
    } else if old.is_empty() {
        // Inserts
        solution.push(DiffOp::Insert(new));
    } else if new.is_empty() {
        // Deletes
        solution.push(DiffOp::Delete(old));
    } else {
        // Divide & Conquer
        let (_shortest_edit_script_len, snake) = find_middle_snake(old, new, vf, vb);
        let (old_a, old_b) = old.split_at(snake.x_start);
        let (new_a, new_b) = new.split_at(snake.y_start);

        conquer(old_a, new_a, vf, vb, solution);
        conquer(old_b, new_b, vf, vb, solution);
    }

    if common_suffix_len > 0 {
        solution.push(common_suffix);
    }
}

pub fn diff<'a>(old: &'a [u8], new: &'a [u8]) -> Vec<ConsolidatedDiffOp> {

    let wrapped_old = WrappedBytes::new(old, ..);
    let wrapped_new = WrappedBytes::new(new, ..);

    let mut solution = Vec::new();

    // The arrays that hold the 'best possible x values' in search from:
    // `vf`: top left to bottom right
    // `vb`: bottom right to top left
    let max_d = max_d(old.len(), new.len());
    let mut vf = V::new(max_d);
    let mut vb = V::new(max_d);

    let mut consolidated_solution = Vec::new();
    conquer(wrapped_old, wrapped_new, &mut vf, &mut vb, &mut solution);
    merge_diff_ops(solution, &mut consolidated_solution);

    consolidated_solution
}

pub fn merge_diff_ops<'a>(solution: Vec<DiffOp<'_>>, result: &mut Vec<ConsolidatedDiffOp>)  {
    for op in solution {
        match op {
            DiffOp::Equal(a, _b) => {
                result.push(ConsolidatedDiffOp::Equal(a.offset(), a.len()));
            },
            DiffOp::Insert(a) => {
                result.push(ConsolidatedDiffOp::Insert(a.clone().dump()));
            },
            DiffOp::Delete(a) => {
                result.push(ConsolidatedDiffOp::Delete(a.offset(), a.len()));
            },
        }
    }

    let old_result = result.clone();
    * result = Vec::new();
    let mut p1 = 0;
    let mut p2 = 1;
    let old_result_len = old_result.len();

    while p1 < old_result_len {
        let op1 = old_result[p1].clone();

        match op1 {
            ConsolidatedDiffOp::Equal(offset1, len1) => {
                let offset = offset1;
                let mut len = len1;

                while p2 < old_result_len {
                    let op2 = old_result[p2].clone();
                    match op2 {
                        ConsolidatedDiffOp::Equal(offset2, len2) => {
                            if offset2 == offset + len {
                                len += len2;
                                p2 += 1;
                            } else {
                                println!("Unexpected");
                                break;
                            }
                        },
                        _ => {
                            break;
                        }
                    }
                }

                result.push(ConsolidatedDiffOp::Equal(offset, len));
            },
            ConsolidatedDiffOp::Insert(v1) => {
                let mut buf = v1.clone();

                while p2 < old_result_len {
                    let op2 = old_result[p2].clone();
                    match op2 {
                        ConsolidatedDiffOp::Insert(v2) => {
                            buf.extend_from_slice(&v2[..]);
                            p2 += 1;
                        },
                        _ => {
                            break;
                        }
                    }
                }

                result.push(ConsolidatedDiffOp::Insert(buf));
            },
            ConsolidatedDiffOp::Delete(offset1, len1) => {
                let offset = offset1;
                let mut len = len1;

                while p2 < old_result_len {
                    let op2 = old_result[p2].clone();
                    match op2 {
                        ConsolidatedDiffOp::Delete(offset2, len2) => {
                            if offset2 == offset + len {
                                len += len2;
                                p2 += 1;
                            } else {
                                break;
                            }
                        },
                        _ => {
                            break;
                        }
                    }
                }

                result.push(ConsolidatedDiffOp::Delete(offset, len));
            },
        }
        p1 = p2;
        p2 += 1;
    }
}

pub fn patch(patch: Vec<ConsolidatedDiffOp>, origin: &[u8]) -> Vec<u8> {
    let mut new = Vec::new();

    for op in patch {
        match op {
            ConsolidatedDiffOp::Equal(offset, len) => {
                new.extend_from_slice(&origin.get(offset..offset + len).unwrap()[..]);
            }
            ConsolidatedDiffOp::Insert(a) => {
                new.extend_from_slice(&a[..]);
            }
            ConsolidatedDiffOp::Delete(_a, _b) => {
                 //
            }
        }
    }

    new
}

#[test]
fn test_diff() {
    macro_rules! random_bytes{
        ($len:expr) => ({
            let mut bytes = [0_u8; $len];
            for byte in bytes.iter_mut() {
                *byte = rand::random::<u8>();
            }
            bytes
        })
    }

    // delete more bytes than insert 
    let mut loops = 3;
    while loops > 0 {
        let mut old = random_bytes!(1000);
        let mut new = random_bytes!(100);

        let res = diff(&mut old[..], &mut new[..]);
        let recovered = patch(res, &old[..]);
        assert_eq!(recovered, new);

        loops -= 1;
    }
    
    loops = 3;
    while loops > 0 {
        let mut old = random_bytes!(1000);
        let mut new = random_bytes!(1000);

        let res = diff(&mut old[..], &mut new[..]);
        let recovered = patch(res, &old[..]);
        assert_eq!(recovered, new);

        loops -= 1;
    }

    loops = 3;
    while loops > 0 {
        let mut old = random_bytes!(100);
        let mut new = random_bytes!(10000);

        let res = diff(&mut old[..], &mut new[..]);
        let recovered = patch(res, &old[..]);
        assert_eq!(recovered, new);

        loops -= 1;
    }
}

#[test]
fn test_common_prefix_length() {
    let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let b = [1, 2, 3, 4, 5, 6,];

    let wrapped_a = WrappedBytes::new(&a[..], ..);
    let wrapped_b = WrappedBytes::new(&b[..], ..);

    assert_eq!(common_prefix_len(wrapped_a, wrapped_b), 6);
}

#[test]
fn test_common_suffix_length() {
    let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let b = [6, 7, 8, 9, 10];

    let wrapped_a = WrappedBytes::new(&a[..], ..);
    let wrapped_b = WrappedBytes::new(&b[..], ..);

    assert_eq!(common_suffix_len(wrapped_a, wrapped_b), 5);
}