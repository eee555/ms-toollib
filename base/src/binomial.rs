use num_bigint::BigUint;
use num_bigint::ToBigUint;
use std::cell::Cell;

pub struct PrimeSieve {
    max: usize,
    composite: Vec<bool>,
}

impl PrimeSieve {
    pub fn new(n: usize) -> Self {
        let max = if n < 2 { 2 } else { n };
        let mut composite = vec![false; max + 1];
        let root_n = (max as f64).sqrt() as usize;
        for i in 2..=root_n {
            if !composite[i] {
                let mut idx = i + i;
                while idx <= max {
                    composite[idx] = true;
                    idx += i;
                }
            }
        }
        PrimeSieve { max, composite }
    }

    pub fn is_prime(&self, n: usize) -> bool {
        if n <= 1 || n > self.max {
            panic!("Prime check out of range: {}", n);
        }
        !self.composite[n]
    }
}

pub struct BinomialEntry {
    pub k: usize,
    pub n: usize,
    pub bco: BigUint,
    pub last_used: Cell<u64>,
}

impl BinomialEntry {
    pub fn new(k: usize, n: usize, bco: BigUint) -> Self {
        BinomialEntry {
            k,
            n,
            bco,
            last_used: Cell::new(0),
        }
    }
}

pub struct BinomialCache {
    cache_size: usize,
    cache_threshold: usize,
    engine: Binomial,
    start: isize,
    use_count: Cell<u64>,
    cache_removal: usize,
    cache: Vec<BinomialEntry>,
    pub cache_hits: u64,
    pub cache_stores: u64,
    pub near_miss: u64,
    pub full_calc: u64,
}

impl BinomialCache {
    pub fn new(cache_size: usize, cache_threshold: usize, engine: Binomial) -> Self {
        BinomialCache {
            cache_size,
            cache_threshold,
            engine,
            start: -1,
            use_count: Cell::new(0),
            cache_removal: cache_size / 2,
            cache: Vec::with_capacity(cache_size),
            cache_hits: 0,
            cache_stores: 0,
            near_miss: 0,
            full_calc: 0,
        }
    }

    pub fn get_binomial(&mut self, k: usize, n: usize) -> BigUint {
        if n <= self.cache_threshold {
            return self.engine.generate(k, n);
        }

        let uc = self.use_count.get() + 1;
        self.use_count.set(uc);

        let mut near_miss_k: Option<usize> = None;
        let mut near_miss_n: Option<usize> = None;
        let mut near_miss_bco: Option<BigUint> = None;

        if self.start >= 0 {
            for i in (0..=self.start as usize).rev() {
                let entry = &self.cache[i];
                if entry.k == k && entry.n == n {
                    self.cache_hits += 1;
                    entry.last_used.set(uc);
                    return entry.bco.clone();
                }
                if entry.n == n && entry.k == k + 1 {
                    near_miss_k = Some(entry.k);
                    near_miss_n = Some(entry.n);
                    near_miss_bco = Some(entry.bco.clone());
                }
            }
        }

        let b = if let (Some(nk), Some(nn), Some(ref bco)) = (near_miss_k, near_miss_n, near_miss_bco) {
            self.near_miss += 1;
            bco * (nk as u64) / ((nn - nk + 1) as u64)
        } else {
            self.full_calc += 1;
            self.engine.generate(k, n)
        };

        if self.start == self.cache_size as isize - 1 {
            self.compress_cache();
        }

        self.start += 1;
        let be = BinomialEntry::new(k, n, b.clone());
        be.last_used.set(uc);
        self.cache.push(be);
        self.cache_stores += 1;

        b
    }

    fn compress_cache(&mut self) {
        eprintln!("Compressing binomial cache");
        let start_idx = self.start as usize;
        let slice = &mut self.cache[..=start_idx];
        slice.sort_by(|a, b| a.last_used.get().cmp(&b.last_used.get()));

        let keep = self.cache_size - self.cache_removal;
        for i in 0..keep {
            let idx = i + self.cache_removal;
            self.cache[i] = BinomialEntry::new(
                self.cache[idx].k,
                self.cache[idx].n,
                self.cache[idx].bco.clone(),
            );
            self.cache[i].last_used.set(self.cache[idx].last_used.get());
        }
        self.start = (self.start - self.cache_removal as isize) as isize;
        self.cache.truncate(self.start as usize + 1);
    }

    pub fn get_max_n(&self) -> usize {
        self.engine.max
    }

    pub fn stats(&self) {
        eprintln!(
            "BinomialCache => stored: {}, hits: {}, near-miss: {}, full-calc: {}",
            self.cache_stores, self.cache_hits, self.near_miss, self.full_calc
        );
    }
}

pub struct Binomial {
    pub max: usize,
    ps: PrimeSieve,
    lookup_limit: usize,
    binomial_lookup: Vec<Vec<Option<BigUint>>>,
}

impl Binomial {
    pub fn new(max: usize, lookup: usize) -> Self {
        let start = std::time::Instant::now();
        let ps = PrimeSieve::new(max);
        let lookup_limit = if lookup < 10 { 10 } else { lookup };
        let lookup2 = lookup_limit / 2;

        let mut binomial_lookup: Vec<Vec<Option<BigUint>>> = Vec::with_capacity(lookup_limit + 1);
        binomial_lookup.push(Vec::new()); // index 0 unused
        for total in 1..=lookup_limit {
            let mut row: Vec<Option<BigUint>> = vec![None; lookup2 + 1];
            let half = total / 2;
            for choose in 0..=half {
                let val = Self::compute_generate(&ps, choose, total, max, lookup_limit, &binomial_lookup);
                row[choose] = Some(val);
            }
            binomial_lookup.push(row);
        }

        eprintln!(
            "Binomial lookup table generated, limit {}, max {}",
            lookup_limit, max
        );
        eprintln!("Took {} ms", start.elapsed().as_millis());

        Binomial {
            max,
            ps,
            lookup_limit,
            binomial_lookup,
        }
    }

    fn compute_generate(
        ps: &PrimeSieve,
        k: usize,
        n: usize,
        max_n: usize,
        lookup_limit: usize,
        lookup: &[Vec<Option<BigUint>>],
    ) -> BigUint {
        if n == 0 && k == 0 {
            return 1u32.to_biguint().unwrap();
        }
        assert!(n >= 1 && n <= max_n, "n must be between 1 and max");
        assert!(k <= n, "k must be <= n");

        let choose = if k <= n - k { k } else { n - k };

        if n <= lookup_limit {
            if let Some(Some(ref val)) = lookup.get(n).and_then(|row| row.get(choose)) {
                return val.clone();
            }
        }

        if choose < 25 {
            Self::combination(choose, n)
        } else {
            Self::combination_large(ps, choose, n)
        }
    }

    pub fn generate(&self, k: usize, n: usize) -> BigUint {
        if n == 0 && k == 0 {
            return 1u32.to_biguint().unwrap();
        }
        if n < 1 || n > self.max {
            panic!(
                "Binomial: expected 1 <= n <= max, got n={}, max={}",
                n, self.max
            );
        }
        if k > n {
            panic!("Binomial: expected k <= n, got k={}, n={}", k, n);
        }

        let choose = if k <= n - k { k } else { n - k };

        if n <= self.lookup_limit {
            if let Some(Some(ref val)) = self.binomial_lookup[n].get(choose) {
                return val.clone();
            }
        }

        if choose < 25 {
            Self::combination(choose, n)
        } else {
            Self::combination_large(&self.ps, choose, n)
        }
    }

    fn combination(mines: usize, squares: usize) -> BigUint {
        let range = if mines <= squares - mines {
            mines
        } else {
            squares - mines
        };

        let mut top = BigUint::from(1u32);
        let mut bot = BigUint::from(1u32);

        for i in 0..range {
            top *= BigUint::from((squares - i) as u64);
            bot *= BigUint::from((i + 1) as u64);
        }

        top / bot
    }

    fn combination_large(ps: &PrimeSieve, mut k: usize, n: usize) -> BigUint {
        if k == 0 || k == n {
            return BigUint::from(1u32);
        }

        let n2 = n / 2;
        if k > n2 {
            k = n - k;
        }
        let nk = n - k;
        let root_n = (n as f64).sqrt() as usize;

        let mut result = BigUint::from(1u32);

        for prime in 2..=n {
            if !ps.is_prime(prime) {
                continue;
            }

            if prime > nk {
                result *= BigUint::from(prime as u64);
                continue;
            }

            if prime > n2 {
                continue;
            }

            if prime > root_n {
                if n % prime < k % prime {
                    result *= BigUint::from(prime as u64);
                }
                continue;
            }

            let mut r = 0usize;
            let mut nn = n;
            let mut kk = k;
            let mut p = 1u64;

            let mut safety = 100;
            while nn > 0 {
                r = if (nn % prime) < (kk % prime + r) { 1 } else { 0 };
                if r == 1 {
                    p = p.saturating_mul(prime as u64);
                }
                nn /= prime;
                kk /= prime;
                safety -= 1;
                if safety < 1 {
                    eprintln!("Safety stop in combination_large!");
                    break;
                }
            }

            if p > 1 {
                result *= BigUint::from(p);
            }
        }

        result
    }
}
