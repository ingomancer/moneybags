use std::{collections::HashMap, io::Write};

mod args;
mod money;

mod moneybag;
use args::{AddCommand, Args, Command, DeleteCommand, EditCommand, ListCommand};
use clap::Parser;
use moneybag::{average_invoice, sum_costs, sum_invoices, Cost, Invoice, Moneybag, Rate};

use money::Money;

fn prompt(prompt: &str) -> String {
    print!("{prompt}");
    std::io::stdout().flush().expect("Could not flush stdout");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Could not read line");
    input.trim().to_string()
}

fn main() {
    let args = Args::parse();
    let filepath = args.file;
    let filepath = shellexpand::tilde(&filepath).to_string();
    let mut moneybag = load_moneybag(&filepath);

    loop {
        let input = prompt("> ");
        let command = match Command::try_parse_from(shlex::split(&input).unwrap()) {
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
                println!("Costs: {costs}\nInvoices: {invoices}\nTotal: {total}\nAverage invoice: {average}");
            } else {
                println!("Costs: {}\nInvoices: {}\nTotal: {}\nAverage invoice: {}\nInvoices left to break even: {}", costs, invoices, total, average, -total/average);
            }
        }
        Command::Save { path } => match path {
            Some(path) => save_moneybag(moneybag, &path),
            None => unreachable!("Path should always be Some"),
        },
        Command::Edit(edit_command) => handle_edit(edit_command, moneybag),
        Command::Delete(delete_command) => handle_delete(delete_command, moneybag),
    }
}

fn handle_delete(delete_command: DeleteCommand, moneybag: &mut Moneybag) {
    match delete_command {
        DeleteCommand::Rate { name } => {
            moneybag.rates.remove(&name);
        }
        DeleteCommand::Invoice { index } => {
            moneybag.invoices.remove(index);
        }
        DeleteCommand::Cost { index } => {
            moneybag.costs.remove(index);
        }
    }
}

fn handle_edit(edit_command: EditCommand, moneybag: &mut Moneybag) {
    match edit_command {
        EditCommand::Rate { name } => edit_rate(&name, moneybag),
        EditCommand::Invoice { index } => edit_invoice(index, moneybag),
        EditCommand::Cost { index } => edit_cost(index, moneybag),
    }
}

fn edit_cost(index: usize, moneybag: &mut Moneybag) {
    let cost = moneybag.costs.get_mut(index).expect("Cost not found");
    let mut input = prompt(&format!("date ({}): ", cost.date));
    if !input.is_empty() {
        cost.date = input;
    }

    cost.amount = loop {
        input = prompt(&format!("amount ({}): ", cost.amount));
        if input.is_empty() {
            break cost.amount;
        }
        if let Ok(amount) = input.parse() {
            break amount;
        }
        println!("Could not parse amount");
    };

    input = prompt(&format!("name ({}): ", cost.name));
    if !input.is_empty() {
        cost.name = input;
    }
}

fn edit_invoice(index: usize, moneybag: &mut Moneybag) {
    let invoice = moneybag.invoices.get_mut(index).expect("Invoice not found");
    let mut input = prompt(&format!("date ({}): ", invoice.date));
    if !input.is_empty() {
        invoice.date = input;
    }

    invoice.amount = loop {
        input = prompt(&format!("amount ({}): ", invoice.amount));
        if input.is_empty() {
            break invoice.amount;
        }
        if let Ok(amount) = input.parse() {
            break amount;
        }
        println!("Could not parse amount");
    };

    if let Some(customer) = &invoice.customer {
        input = prompt(&format!("customer ({customer}): "));
    } else {
        input = prompt("customer: ");
    }
    if !input.is_empty() {
        invoice.customer = Some(input);
    }

    if let Some(rate) = &invoice.rate {
        input = prompt(&format!("rate ({}): ", rate.rate));
    } else {
        input = prompt("rate: ");
    }
    if !input.is_empty() {
        if moneybag.rates.contains_key(&input) {
            invoice.rate = Some(*moneybag.rates.get(&input).unwrap());
        } else {
            println!("Rate {input} not found in rates");
        }
    }
}

fn edit_rate(name: &str, moneybag: &mut Moneybag) {
    let rate = moneybag.rates.get_mut(name).expect("Rate not found");

    rate.rate = loop {
        let input = prompt(&format!("rate ({}): ", rate.rate));
        if input.is_empty() {
            break rate.rate;
        }
        if let Ok(rate) = input.parse() {
            break rate;
        }
        println!("Could not parse rate");
    };
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
