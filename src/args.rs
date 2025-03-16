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
    #[clap(subcommand, alias = "a")]
    Add(AddCommand),
    #[clap(subcommand, alias = "l")]
    List(ListCommand),
    #[clap(subcommand, alias = "e")]
    Edit(EditCommand),
    #[clap(subcommand, alias = "d")]
    Delete(DeleteCommand),

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
    /// Add a cost
    #[clap(alias = "c")]
    Cost {
        date: String,
        amount: Money,
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EditCommand {
    #[clap(alias = "r")]
    Rate { name: String },
    #[clap(alias = "i")]
    Invoice { index: usize },
    #[clap(alias = "c")]
    Cost { index: usize },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DeleteCommand {
    #[clap(alias = "r")]
    Rate { name: String },
    #[clap(alias = "i")]
    Invoice { index: usize },
    #[clap(alias = "c")]
    Cost { index: usize },
}
