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

use std::{collections::HashMap, io::Write};

use clap::{Parser, Subcommand};

mod money;

mod moneybag;
use moneybag::{average_invoice, sum_costs, sum_invoices, Cost, Invoice, Moneybag, Rate};

use money::Money;
#[derive(Debug, Parser)]
struct Args {
    /// File to store data in
    #[arg(short, long, default_value = "~/.moneybags")]
    file: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[clap(subcommand, alias = "a")]
    Add(AddCommand),
    #[clap(subcommand, alias = "s")]
    Show(ShowCommand),

    /// Calculate difference between costs and invoices
    #[clap(alias = "b")]
    Balance,
}

#[derive(Debug, Subcommand)]
enum ShowCommand {
    /// List hourly rates
    #[clap(alias = "r")]
    Rates,
    /// List invoices
    #[clap(alias = "i")]
    Invoices,
    /// List costs
    #[clap(alias = "c")]
    Costs,
}

#[derive(Debug, Subcommand)]
enum AddCommand {
    /// Add an hourly rate, with a name
    Rate { rate: Money, name: String },
    #[clap(alias = "r")]
    /// Add an invoice, with a date and amount. If a rate is given, assumes amount to be hours and calculates total.
    #[clap(alias = "i")]
    Invoice {
        date: String,
        amount: Money,
        #[clap(short, long)]
        rate: Option<String>,
        #[clap(short, long)]
        customer: Option<String>,
    },
    /// Add a cost
    #[clap(alias = "c")]
    Cost {
        date: String,
        amount: Money,
        name: String,
    },
}

fn main() {
    let args = Args::parse();
    let filepath = args.file;
    let filepath = shellexpand::tilde(&filepath).to_string();
    let mut moneybag = load_moneybag(&filepath);

    handle_command(args.command, &mut moneybag);

    save_moneybag(&moneybag, &filepath);
}

fn load_moneybag(filepath: &String) -> Moneybag {
    if let Ok(json) = std::fs::read_to_string(filepath) {
        serde_json::from_str(&json).expect("Could not parse file as a moneybag")
    } else {
        Moneybag {
            invoices: vec![],
            rates: HashMap::new(),
            costs: vec![],
        }
    }
}

fn save_moneybag(moneybag: &Moneybag, filepath: &str) {
    let json = serde_json::to_string_pretty(&moneybag)
        .unwrap_or_else(|_| panic!("Could not serialize moneybag. Contents: {:?}", moneybag));
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
        .expect("Could not open file for writing");
    file.write_all(json.as_bytes())
        .expect("Could not write to file");
}

fn handle_command(command: Command, moneybag: &mut Moneybag) {
    match command {
        Command::Add(add_command) => handle_add(add_command, moneybag),
        Command::Show(show_command) => handle_show(show_command, moneybag),
        Command::Balance => {
            let costs = sum_costs(&moneybag.costs);
            let invoices = sum_invoices(&moneybag.invoices);
            let average = average_invoice(&moneybag.invoices);
            let total = invoices - costs;

            println!("Costs: {}\nInvoices: {}\nTotal: {}\nAverage invoice: {}\nInvoices left to break even: {}", costs, invoices, total, average, -total/average);
        }
    }
}

fn handle_add(add_command: AddCommand, moneybag: &mut Moneybag) {
    match add_command {
        AddCommand::Rate { rate, name } => {
            moneybag.rates.insert(name, Rate { rate });
        }
        AddCommand::Invoice {
            date,
            amount,
            rate,
            customer,
        } => {
            if let Some(rate) = &rate {
                if !moneybag.rates.contains_key(rate) {
                    println!("Rate {rate} not found in rates");
                } else {
                    let rate = moneybag.rates.get(rate).unwrap();
                    moneybag.invoices.push(Invoice {
                        date,
                        amount,
                        customer,
                        rate: Some(*rate),
                    });
                }
            } else {
                moneybag.invoices.push(Invoice {
                    date,
                    amount,
                    customer,
                    rate: None,
                });
            }
        }
        AddCommand::Cost { date, amount, name } => {
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
    }
}

fn handle_show(show_command: ShowCommand, moneybag: &Moneybag) {
    match show_command {
        ShowCommand::Rates => {
            for (name, rate) in &moneybag.rates {
                println!("{}: {}", name, rate.rate);
            }
        }
        ShowCommand::Invoices => {
            for invoice in &moneybag.invoices {
                println!("{}", invoice)
            }
        }
        ShowCommand::Costs => {
            for cost in &moneybag.costs {
                println!("{} {} {}", cost.date, cost.amount, cost.name);
            }
        }
    }
}
