//! Small seeded PRNG (xorshift64*). A seed is enough to reproduce an order,
//! so shuffling is testable and needs no `getrandom`/JS dependency.

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // A zero state would stay zero forever.
        Rng((seed ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// 0..n (n > 0)
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Fisher–Yates
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_order() {
        let mut a: Vec<u32> = (0..20).collect();
        let mut b = a.clone();
        Rng::new(7).shuffle(&mut a);
        Rng::new(7).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    }

    #[test]
    fn zero_seed_works() {
        let mut r = Rng::new(0x9E37_79B9_7F4A_7C15);
        assert_ne!(r.next_u64(), r.next_u64());
    }
}
