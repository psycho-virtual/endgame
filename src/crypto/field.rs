// src/crypto/field.rs

use rand::random;
use std::ops::{Add, Div, Mul, Sub};

pub const FIELD_PRIME: u64 = 2_147_483_647;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FieldElement {
    value: u64,
}

impl FieldElement {
    pub fn new(value: u64) -> Self {
        FieldElement {
            value: value % FIELD_PRIME,
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn zero() -> Self {
        FieldElement { value: 0 }
    }

    pub fn one() -> Self {
        FieldElement { value: 1 }
    }

    pub fn random() -> Self {
        FieldElement::new(random::<u64>())
    }

    pub fn pow(&self, exp: usize) -> Self {
        if exp == 0 {
            return Self::one();
        }
        let mut result = *self;
        let mut exp = exp;
        let mut base = *self;

        while exp > 1 {
            if exp % 2 == 1 {
                result = result * base;
            }
            base = base * base;
            exp /= 2;
        }
        if exp == 1 {
            result = result * base;
        }
        result
    }

    pub fn inverse(&self) -> Option<Self> {
        if self.value == 0 {
            return None;
        }
        // Using Fermat's little theorem: a^(p-1) ≡ 1 (mod p)
        // Therefore, a^(p-2) is the multiplicative inverse
        Some(self.pow((FIELD_PRIME - 2) as usize))
    }
}

// Add From<u64> implementation
impl From<u64> for FieldElement {
    fn from(value: u64) -> Self {
        FieldElement::new(value)
    }
}

// Implement remaining operator traits
impl Add for FieldElement {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum = self.value + other.value;
        FieldElement::new(if sum >= FIELD_PRIME {
            sum - FIELD_PRIME
        } else {
            sum
        })
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let diff = if self.value >= other.value {
            self.value - other.value
        } else {
            FIELD_PRIME - (other.value - self.value)
        };
        FieldElement::new(diff)
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        FieldElement::new((self.value as u128 * other.value as u128 % FIELD_PRIME as u128) as u64)
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if let Some(inv) = other.inverse() {
            self * inv
        } else {
            panic!("Division by zero")
        }
    }
}
