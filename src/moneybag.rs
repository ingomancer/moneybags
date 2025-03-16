use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{money, Money};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Invoice {
    pub(crate) date: String,
    pub(crate) amount: Money,
    pub(crate) rate: Option<Rate>,
    pub(crate) customer: Option<String>,
}

impl Display for Invoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let amount = match self.rate {
            Some(rate) => format!(
                "{} ({} * {})",
                rate.rate * self.amount,
                self.amount,
                rate.rate,
            ),
            None => format!("{}", self.amount),
        };
        if self.customer.is_some() {
            write!(
                f,
                "{}: {} ({})",
                self.date,
                amount,
                self.customer.as_ref().unwrap()
            )
        } else {
            write!(f, "{}: {}", self.date, amount)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub(crate) struct Rate {
    pub(crate) rate: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Cost {
    pub(crate) date: String,
    pub(crate) amount: Money,
    pub(crate) name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Moneybag {
    pub(crate) invoices: Vec<Invoice>,
    pub(crate) rates: HashMap<String, Rate>,
    pub(crate) costs: Vec<Cost>,
}

pub(crate) fn sum_costs(costs: &[Cost]) -> Money {
    costs.iter().map(|cost| cost.amount).sum()
}

pub(crate) fn sum_invoices(invoices: &[Invoice]) -> Money {
    invoices
        .iter()
        .map(|invoice| {
            if let Some(rate) = invoice.rate {
                invoice.amount * rate.rate
            } else {
                invoice.amount
            }
        })
        .sum()
}

pub(crate) fn average_invoice(invoices: &[Invoice]) -> Money {
    let invoice_count = i64::try_from(invoices.len())
        .unwrap_or_else(|_| panic!("Having more than {} invoices is not supported", i64::MAX));
    if invoice_count != 0 {
        sum_invoices(invoices) / invoice_count
    } else {
        money::Money::default()
    }
}
