use std::usize;
struct Primes {
    limit: usize,
}

impl Primes {
    fn iter(&self) -> PrimesIter {
        PrimesIter {
            index: 2,
            computed: compute_primes(self.limit),
        }
    }
    fn new(limit: usize) -> Primes {
        Primes { limit }
    }
}

// 计算在limit之内的素数并放在一个Vec中
fn compute_primes(limit: usize) -> Vec<bool> {
    let mut sieve = vec![true; limit];
    let mut m = 2;
    while m * m < limit {
        if sieve[m] {
            for i in (m * 2..limit).step_by(m) {
                sieve[i] = false;
            }
        }
        m += 1;
    }
    sieve
}

struct PrimesIter {
    index: usize,
    computed: Vec<bool>,
}
impl Iterator for PrimesIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.index += 1;
            if self.index > self.computed.len() - 1 {
                return None;
            } else if self.computed[self.index] {
                return Some(self.index);
            } else {
                continue;
            }
        }
    }
}

fn useage() -> Vec<usize> {
    let primes = Primes::new(100);
    let mut res: Vec<usize> = vec![];
    for i in primes.iter() {
        res.push(i);
    }
    res
}

#[cfg(test)]
pub mod tests {
    use super::useage;

    #[test]
    fn test_custom_iter() {
        let res: Vec<usize> = useage();
        let expect: Vec<usize> = vec![
            3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];
        assert_eq!(res, expect)
    }
}
