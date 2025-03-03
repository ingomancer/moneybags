use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct Money {
    amount: i64,
}

impl Money {
    pub fn is_zero(self) -> bool {
        self.amount == 0
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Self) -> Self::Output {
        Money {
            amount: self.amount - rhs.amount,
        }
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Self::Output {
        Money {
            amount: -self.amount,
        }
    }
}

impl Div for Money {
    type Output = Money;
    fn div(self, rhs: Self) -> Self::Output {
        Money {
            amount: (self.amount * 100) / rhs.amount,
        }
    }
}

impl Div<i64> for Money {
    type Output = Money;
    fn div(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount / rhs,
        }
    }
}

impl Mul for Money {
    type Output = Money;
    fn mul(self, rhs: Self) -> Self::Output {
        Money {
            amount: (self.amount * rhs.amount) / 100,
        }
    }
}

impl Mul<i64> for Money {
    type Output = Money;
    fn mul(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount * rhs,
        }
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Self) -> Self::Output {
        Money {
            amount: self.amount + rhs.amount,
        }
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money { amount: 0 }, |a, b| a + b)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:0>2}", self.amount / 100, (self.amount % 100).abs())
    }
}

impl FromStr for Money {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount = s
            .split('.')
            .map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Money {
            amount: amount[0] * 100 + if amount.len() == 2 { amount[1] } else { 0 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money() {
        let a = Money { amount: 1000 };
        let b = Money { amount: 2000 };
        assert_eq!(a + b, Money { amount: 3000 });
        assert_eq!(a - b, Money { amount: -1000 });
        assert_eq!(a * b, Money { amount: 20000 });
        assert_eq!(a / b, Money { amount: 50 });
        assert_eq!(-a, Money { amount: -1000 });
        assert_eq!(a / 2, Money { amount: 500 });
        assert_eq!(b / 2, Money { amount: 1000 });
        assert_eq!(a * 2, Money { amount: 2000 });
    }

    #[test]
    fn test_money_display() {
        let a = Money { amount: 1000 };
        assert_eq!(format!("{a}"), "10.00");
        let b = Money { amount: 2001 };
        assert_eq!(format!("{b}"), "20.01");
        let c = Money { amount: 200 };
        assert_eq!(format!("{c}"), "2.00");
        let d = Money { amount: -153 };
        assert_eq!(format!("{d}"), "-1.53");
    }

    #[test]
    fn test_money_from_str() {
        let a = Money::from_str("10.00").unwrap();
        assert_eq!(a, Money { amount: 1000 });
        let b = Money::from_str("20.01").unwrap();
        assert_eq!(b, Money { amount: 2001 });
        let c = Money::from_str("2.00").unwrap();
        assert_eq!(c, Money { amount: 200 });
    }
}
