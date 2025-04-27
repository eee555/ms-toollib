use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

#[derive(Clone, Debug)]
pub struct BigNumber {
    // 科学计数法表示的大数字
    // 必定大于等于1，a必定满足小于10大于等于1
    pub a: f64,
    pub b: i32,
}

impl Mul for &BigNumber {
    type Output = BigNumber;
    fn mul(self, other: &BigNumber) -> Self::Output {
        if self.a == 0.0 || other.a == 0.0 {
            return BigNumber { a: 0.0, b: 0 };
        }
        let mut new_a = self.a * other.a;
        let mut new_b = self.b + other.b;
        if new_a >= 10.0 {
            new_a = new_a / 10.0;
            new_b += 1;
        }
        BigNumber { a: new_a, b: new_b }
    }
}

impl Mul<f64> for &BigNumber {
    type Output = BigNumber;
    fn mul(self, other: f64) -> Self::Output {
        if self.a == 0.0 || other == 0.0 {
            return BigNumber { a: 0.0, b: 0 };
        }
        let mut new_a = self.a * other as f64;
        let mut new_b = self.b;
        while new_a >= 10.0 {
            new_a /= 10.0;
            new_b += 1;
        }
        while new_a < 1.0 {
            new_a *= 10.0;
            new_b -= 1;
        }
        BigNumber { a: new_a, b: new_b }
    }
}

impl Add for &BigNumber {
    type Output = BigNumber;
    fn add(self, other: &BigNumber) -> Self::Output {
        let (larger, smaller) = if self.b > other.b {
            (self, other)
        } else {
            (other, self)
        };
        let diff = (larger.b - smaller.b) as u32;
        let mut new_a = larger.a + smaller.a / (10.0f64.powi(diff as i32));
        let mut new_b = larger.b;
        while new_a >= 10.0 {
            new_a /= 10.0;
            new_b += 1;
        }
        BigNumber { a: new_a, b: new_b }
    }
}

// BigNumber 与 BigNumber 相除
impl Div for &BigNumber {
    type Output = BigNumber;
    fn div(self, other: &BigNumber) -> Self::Output {
        if self.a == 0.0 {
            return BigNumber { a: 0.0, b: 0 };
        }
        let new_a = self.a / other.a;
        let new_b = self.b - other.b;
        let mut result = BigNumber { a: new_a, b: new_b };
        while result.a >= 10.0 {
            result.a /= 10.0;
            result.b += 1;
        }
        while result.a < 1.0 {
            result.a *= 10.0;
            result.b -= 1;
        }
        result
    }
}

// BigNumber 与 BigNumber 复合赋值加法
impl AddAssign<&BigNumber> for BigNumber {
    fn add_assign(&mut self, other: &BigNumber) {
        if self.b > other.b {
            let diff = (self.b - other.b) as f64;
            self.a += other.a / (10.0f64).powi(diff as i32);
        } else if self.b < other.b {
            let diff = (other.b - self.b) as f64;
            self.a = other.a + self.a / (10.0f64).powi(diff as i32);
            self.b = other.b;
        } else {
            self.a += other.a;
        }

        while self.a >= 10.0 {
            self.a /= 10.0;
            self.b += 1;
        }
    }
}

impl DivAssign<&BigNumber> for BigNumber {
    fn div_assign(&mut self, other: &BigNumber) {
        if self.a == 0.0 {
            return;
        }
        self.a /= other.a;
        self.b -= other.b;

        while self.a < 1.0 {
            self.a *= 10.0;
            self.b -= 1;
        }
    }
}

impl MulAssign<&BigNumber> for BigNumber {
    fn mul_assign(&mut self, other: &BigNumber) {
        if self.a == 0.0 || other.a == 0.0 {
            self.a = 0.0;
            self.b = 0;
            return;
        }
        self.a *= other.a;
        self.b += other.b;

        while self.a >= 10.0 {
            self.a /= 10.0;
            self.b += 1;
        }
    }
}

impl std::ops::MulAssign<f64> for BigNumber {
    fn mul_assign(&mut self, other: f64) {
        if self.a == 0.0 || other == 0.0 {
            self.a = 0.0;
            self.b = 0;
            return;
        }
        self.a *= other;
        while self.a >= 10.0 {
            self.a /= 10.0;
            self.b += 1;
        }

        while self.a < 1.0 {
            self.a *= 10.0;
            self.b -= 1;
        }
    }
}

impl Into<f64> for BigNumber {
    fn into(self) -> f64 {
        self.a * 10.0_f64.powi(self.b)
    }
}
