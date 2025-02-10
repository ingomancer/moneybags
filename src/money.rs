use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct Money {
    amount: f64,
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
            amount: (self.amount / rhs.amount) * 100.0,
        }
    }
}

impl Div<f64> for Money {
    type Output = Money;
    fn div(self, rhs: f64) -> Self::Output {
        Money {
            amount: self.amount / rhs,
        }
    }
}

impl Mul for Money {
    type Output = Money;
    fn mul(self, rhs: Self) -> Self::Output {
        Money {
            amount: (self.amount * rhs.amount) / 100.0,
        }
    }
}

impl Mul<f64> for Money {
    type Output = Money;
    fn mul(self, rhs: f64) -> Self::Output {
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
        iter.fold(Money { amount: 0.0 }, |a, b| a + b)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.amount / 100.0)
    }
}

impl FromStr for Money {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount = s
            .split('.')
            .map(|part| part.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Money {
            amount: amount[0] * 100.0 + if amount.len() == 2 { amount[1] } else { 0.0 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money() {
        let a = Money { amount: 1000.0 };
        let b = Money { amount: 2000.0 };
        assert_eq!(a + b, Money { amount: 3000.0 });
        assert_eq!(a - b, Money { amount: -1000.0 });
        assert_eq!(a * b, Money { amount: 20000.0 });
        assert_eq!(a / b, Money { amount: 50.0 });
        assert_eq!(-a, Money { amount: -1000.0 });
        assert_eq!(a / 2.0, Money { amount: 500.0 });
        assert_eq!(b / 2.0, Money { amount: 1000.0 });
        assert_eq!(a * 2.0, Money { amount: 2000.0 });
    }

    #[test]
    fn test_money_display() {
        let a = Money { amount: 1000.0 };
        assert_eq!(format!("{}", a), "10.00");
        let b = Money { amount: 2001.0 };
        assert_eq!(format!("{}", b), "20.01");
        let c = Money { amount: 200.0 };
        assert_eq!(format!("{}", c), "2.00");
    }

    #[test]
    fn test_money_from_str() {
        let a = Money::from_str("10.00").unwrap();
        assert_eq!(a, Money { amount: 1000.0 });
        let b = Money::from_str("20.01").unwrap();
        assert_eq!(b, Money { amount: 2001.0 });
        let c = Money::from_str("2.00").unwrap();
        assert_eq!(c, Money { amount: 200.0 });
    }
}
