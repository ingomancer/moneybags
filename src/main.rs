/*

Commandline yearly costs vs invoicing tracker.

Stores information in some sort of reasonable format, and allows you to look
 at the costs of the year, and the invoices that have been paid.

Can keep track of hourly rates, so invoices can be entered either as a sum, or as hours to invoice.
Costs can be entered as one-offs, or as monthly costs.

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

    #[arg(short, long, default_value_t = false)]
    autosave: bool,
}

#[derive(Debug, Parser)]
#[command(
    multicall = true,
    disable_help_subcommand = true,
    disable_help_flag = true
)]
enum Command {
    #[clap(subcommand, alias = "a")]
    Add(AddCommand),
    #[clap(subcommand, alias = "l")]
    List(ListCommand),

    Save {
        path: Option<String>,
    },

    /// Calculate difference between costs and invoices
    #[clap(alias = "b")]
    Balance,
}

#[derive(Debug, Subcommand)]
enum ListCommand {
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
    #[clap(alias = "r")]
    Rate { rate: Money, name: String },
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

    loop {
        print!("> ");
        std::io::stdout().flush().expect("Could not flush stdout");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Could not read line");
        let command = match Command::try_parse_from(shlex::split(input.trim()).unwrap()) {
            Ok(command) => match command {
                Command::Save { path: None } => Command::Save {
                    path: Some(filepath.clone()),
                },
                _ => command,
            },
            Err(e) => {
                println!("{e}");
                continue;
            }
        };
        handle_command(command, &mut moneybag);
        if args.autosave {
            save_moneybag(&moneybag, &filepath);
        }
    }
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
        .unwrap_or_else(|_| panic!("Could not serialize moneybag. Contents: {moneybag:?}"));
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
        Command::List(list_command) => handle_list(&list_command, moneybag),
        Command::Balance => {
            let costs = sum_costs(&moneybag.costs);
            let invoices = sum_invoices(&moneybag.invoices);
            let average = average_invoice(&moneybag.invoices);
            let total = invoices - costs;
            if average.is_zero() {
                println!(
                    "Costs: {costs}\nInvoices: {invoices}\nTotal: {total}\nAverage invoice: {average}"
                );
            } else {
                println!("Costs: {}\nInvoices: {}\nTotal: {}\nAverage invoice: {}\nInvoices left to break even: {}", costs, invoices, total, average, -total/average);
            }
        }
        Command::Save { path } => match path {
            Some(path) => save_moneybag(moneybag, &path),
            None => unreachable!("Path should always be Some"),
        },
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
                if moneybag.rates.contains_key(rate) {
                    let rate = moneybag.rates.get(rate).unwrap();
                    moneybag.invoices.push(Invoice {
                        date,
                        amount,
                        customer,
                        rate: Some(*rate),
                    });
                } else {
                    println!("Rate {rate} not found in rates");
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
                        date: format!("2025-{month:02}"),
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

fn handle_list(list_command: &ListCommand, moneybag: &Moneybag) {
    match list_command {
        ListCommand::Rates => {
            for (name, rate) in &moneybag.rates {
                println!("{}: {}", name, rate.rate);
            }
        }
        ListCommand::Invoices => {
            for (i, invoice) in moneybag.invoices.iter().enumerate() {
                println!("{i}: {invoice}");
            }
        }
        ListCommand::Costs => {
            for (i, cost) in moneybag.costs.iter().enumerate() {
                println!("{i}: {} {} {}", cost.date, cost.amount, cost.name);
            }
        }
    }
}
