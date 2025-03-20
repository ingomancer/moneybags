# Introduction

Very basic commandline yearly costs vs invoicing tracker.

Stores information in some sort of reasonable format, and allows you to look
 at the costs of the year, and the invoices that have been paid.

Can keep track of hourly rates, so invoices can be entered either as a sum, or as hours to invoice.
Costs can be entered as one-offs, or as monthly costs.

## Use

Moneybags runs as an interactive shell.

To start it from the repo, simply run ```cargo run```.

The main executable takes two options:

```--file```, letting you specify where to save your stuff (default: ~/.moneybags). If you're on a system where ~ doesn't mean anything or HOME isn't set, I don't know what happens. Please let me know!

```--autosave```, which causes it to save after every change, instead of requiring a manual save command. Might be reasonable to have on by default? It's off by default now because I keep doing weird stuff while developing.

After starting, you will be met by a prompt, where you can for eample write help
```
> help
Usage: <COMMAND>

Commands:
  add      
  list     
  edit     
  delete   
  save     
  balance  Calculate difference between costs and invoices
  help     Print this message or the help of the given subcommand(s)
```

Here are some small examples. With a new file:
```
> balance
Costs: 0.00
Invoices: 0.00
Total: 0.00
Average invoice: 0.00

> add rate 900 hourly

> add cost monthly 50000 wages

> add invoice 2025-0131 150 --rate hourly

> list invoices
0: 2025-01-31: 135000.00 (150.00 * 900.00)

> list costs
0: 2025-01 50000.00 wages
1: 2025-02 50000.00 wages
...
10: 2025-11 50000.00 wages
11: 2025-12 50000.00 wages

> balance
Costs: 600000.00
Invoices: 135000.00
Total: -465000.00
Average invoice: 135000.00
Invoices left to break even: 3.44