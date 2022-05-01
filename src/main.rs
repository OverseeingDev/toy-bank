mod bank;
mod fixedpoint;
mod transactions;

use clap::Parser;
use std::path::PathBuf;
use transactions::csv_to_transaction_iterator;

use crate::bank::BankDatabase;

#[derive(Parser, Debug)]
struct Args {
    transactions_filepath: PathBuf,
}

fn main() {
    let transactions_filepath = Args::parse().transactions_filepath;
    let mut bank = BankDatabase::default();

    for transaction in csv_to_transaction_iterator(transactions_filepath) {
        bank.execute_transaction(transaction);
    }

    println!("{}", bank);
}
