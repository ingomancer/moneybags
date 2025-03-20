use clap::{Parser, Subcommand};

use crate::Money;

#[derive(Debug, Parser)]
pub(crate) struct Args {
    /// File to store data in
    #[arg(short, long, default_value = "~/.moneybags")]
    pub(crate) file: String,

    #[arg(short, long, default_value_t = false)]
    pub(crate) autosave: bool,
}

#[derive(Debug, Parser)]
#[command(multicall = true, disable_help_flag = true)]
pub(crate) enum Command {
    /// Add a rate, invoice, or cost
    #[clap(subcommand, alias = "a")]
    Add(AddCommand),
    /// List rates, invoices, or costs
    #[clap(subcommand, alias = "l")]
    List(ListCommand),
    /// Interactively edit a rate, invoice, or cost
    #[clap(subcommand, alias = "e")]
    Edit(EditCommand),
    /// Delete a rate, invoice, or cost
    #[clap(subcommand, alias = "d")]
    Delete(DeleteCommand),
    /// Write pending changes to file. There is currently no way to see pending changes
    #[clap(alias = "s")]
    Save { path: Option<String> },

    /// Calculate difference between costs and invoices
    #[clap(alias = "b")]
    Balance,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ListCommand {
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
pub(crate) enum AddCommand {
    /// Add an hourly rate, with a name
    #[clap(alias = "r")]
    Rate { rate: Money, name: String },
    /// Add an invoice, with a date and amount. If a rate is given, assumes amount
    /// to be hours and calculates total.
    #[clap(alias = "i")]
    Invoice {
        date: String,
        amount: Money,
        #[clap(short, long)]
        rate: Option<String>,
        #[clap(short, long)]
        customer: Option<String>,
    },
    /// Add a cost. If date is "monthly", an entry will be generated for each month.
    #[clap(alias = "c")]
    Cost {
        date: String,
        amount: Money,
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EditCommand {
    /// Edit a rate, identified by name
    #[clap(alias = "r")]
    Rate { name: String },
    /// Edit an invoice, identified by index (see list)
    #[clap(alias = "i")]
    Invoice { index: usize },
    /// Edit a cost, identified by index (see list)
    #[clap(alias = "c")]
    Cost { index: usize },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DeleteCommand {
    /// Delete a rate, identified by name
    #[clap(alias = "r")]
    Rate { name: String },
    /// Delete an invoice, identified by index (see list)
    #[clap(alias = "i")]
    Invoice { index: usize },
    /// Delete a cost, identified by index (see list)
    #[clap(alias = "c")]
    Cost { index: usize },
}
