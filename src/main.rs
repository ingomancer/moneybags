/*

Commandline yearly costs vs invoicing tracker.

Stores information in some sort of reasonable format, and allows you to look
 at the costs of the year, and the invoices that have been paid.

Can keep track of hourly rates, so invoices can be entered either as a sum, or as hours to invoice.
Costs can be entered as one-offs, or as monthly costs.


Example usage:
moneybags add-rate 765 hourly
moneybags add-rate 900 on-call

moneybags add-invoice 2025-01-31 1000
moneybags add-invoice 2025-01-31 50 hourly
moneybags add-invoice 2025-01-31 10 on-call

moneybags show-invoices
-> YTD: 48250
-> 2025-01-31 1000
-> 2025-01-31 38250 (50 * 765)
-> 2025-01-31 9000 (10 * 900)

moneybags add-cost 2025-01 10000 insurance
moneybags add-cost monthly 5000 wages

moneybags show-costs
-> Total: 70000
2025-01 10000 insurance
2025-01 5000 wages
2025-02 5000 wages
2025-03 5000 wages
2025-04 5000 wages
2025-05 5000 wages
2025-06 5000 wages
2025-07 5000 wages
2025-08 5000 wages
2025-09 5000 wages
2025-10 5000 wages
2025-11 5000 wages
2025-12 5000 wages

moneybags balance
-> Total: −21750
*/

use std::{collections::HashMap, fmt::Display, io::Write};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct Args {
    /// File to store data in
    #[arg(short, long, default_value = "~/.moneybags")]
    file: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add an hourly rate, with a name
    AddRate { rate: f64, name: String },
    /// List hourly rates
    ShowRates,
    /// Add an invoice, with a date and amount. If a rate is given, assumes amount to be hours and calculates total.
    AddInvoice {
        date: String,
        amount: f64,
        rate: Option<String>,
    },
    /// List invoices
    ShowInvoices,
    /// Add a cost
    AddCost {
        date: String,
        amount: f64,
        name: String,
    },
    /// List costs
    ShowCosts,
    /// Calculate difference between costs and invoices
    Balance,
}

#[derive(Debug, Serialize, Deserialize)]
struct Invoice {
    date: String,
    amount: f64,
}

impl Display for Invoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.date, self.amount)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Rate {
    rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cost {
    date: String,
    amount: f64,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Moneybag {
    invoices: Vec<Invoice>,
    rates: HashMap<String, Rate>,
    costs: Vec<Cost>,
}

fn sum_costs(costs: &[Cost]) -> f64 {
    costs.iter().map(|cost| cost.amount).sum()
}

fn sum_invoices(invoices: &[Invoice]) -> f64 {
    invoices.iter().map(|invoice| invoice.amount).sum()
}

fn average_invoice(invoices: &[Invoice]) -> f64 {
    sum_invoices(invoices) / invoices.len() as f64
}

fn main() {
    let args = Args::parse();
    let filepath = args.file.unwrap();
    let filepath = shellexpand::tilde(&filepath).to_string();
    let mut moneybag;
    if let Ok(json) = std::fs::read_to_string(&filepath) {
        moneybag = serde_json::from_str(&json).expect("Could not parse file as a moneybag");
    } else {
        moneybag = Moneybag {
            invoices: vec![],
            rates: HashMap::new(),
            costs: vec![],
        };
    }
    match args.command {
        Command::AddRate { rate, name } => {
            moneybag.rates.insert(name, Rate { rate });
        }
        Command::ShowRates => {
            for (name, rate) in &moneybag.rates {
                println!("{}: {}", name, rate.rate);
            }
        }

        Command::AddInvoice { date, amount, rate } => {
            if let Some(rate) = &rate {
                if !moneybag.rates.contains_key(rate) {
                    println!("Rate {rate} not found in rates");
                    return;
                } else {
                    let rate = moneybag.rates.get(rate).unwrap().rate;
                    moneybag.invoices.push(Invoice {
                        date,
                        amount: rate * amount,
                    });
                }
            } else {
                moneybag.invoices.push(Invoice { date, amount });
            }
        }
        Command::ShowInvoices => {
            for invoice in &moneybag.invoices {
                println!("{}", invoice)
            }
        }
        Command::AddCost { date, amount, name } => {
            if date == "monthly" {
                for month in 1..=12 {
                    moneybag.costs.push(Cost {
                        date: format!("2025-{:02}", month),
                        amount,
                        name: name.clone(),
                    });
                }
            } else {
                moneybag.costs.push(Cost { date, amount, name });
            }
        }
        Command::ShowCosts => {
            for cost in &moneybag.costs {
                println!("{} {} {}", cost.date, cost.amount, cost.name);
            }
        }
        Command::Balance => {
            let costs = sum_costs(&moneybag.costs);
            let invoices = sum_invoices(&moneybag.invoices);
            let average = average_invoice(&moneybag.invoices);
            let total = invoices - costs;

            println!("Costs: {}\nInvoices: {}\nTotal: {}\nAverage invoice: {}\nInvoices left to break even: {}", costs, invoices, total, average, -total/average);
        }
    }

    let json = serde_json::to_string(&moneybag).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
        .unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
