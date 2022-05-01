use std::path::PathBuf;

use serde::Deserialize;

use crate::fixedpoint::string_to_fixed_point;

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    DEPOSIT,
    WITHDRAWAL,
    DISPUTE,
    RESOLVE,
    CHARGEBACK,
}

#[derive(Debug, Deserialize)]
struct DeserializedTransaction {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: String,
}

#[derive(Debug, Copy, Clone)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub amount: i64,
}

pub type TransactionIdTuple = (u32, Transaction);

impl TryFrom<DeserializedTransaction> for TransactionIdTuple {
    type Error = &'static str;
    fn try_from(deserialized: DeserializedTransaction) -> Result<Self, Self::Error> {
        let amount = string_to_fixed_point(&deserialized.amount)?;
        Ok((
            deserialized.tx,
            Transaction {
                amount,
                r#type: deserialized.r#type,
                client: deserialized.client,
            },
        ))
    }
}

pub fn csv_to_transaction_iterator(path: PathBuf) -> impl Iterator<Item = TransactionIdTuple> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .expect("Cannot read from file");

    reader
        .into_deserialize()
        .filter_map(|e: Result<DeserializedTransaction, csv::Error>| {
            e.ok().and_then(|i: DeserializedTransaction| {
                let result = i.try_into();
                match result {
                    Err(msg) => {
                        eprintln!("Warning: dropped transaction: {}", msg);
                        None
                    }
                    Ok(res) => Some(res),
                }
            })
        })
}
