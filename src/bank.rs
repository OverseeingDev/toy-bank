use crate::fixedpoint::fixed_point_to_string;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
};

use crate::transactions::{Transaction, TransactionIdTuple, TransactionType::*};
/**
 * Derived default makes sense here,
 * funds all zeroed and not locked.
 */
#[derive(Default)]
struct Account {
    available_funds: i64,
    held_funds: i64,
    locked: bool,
}

impl Account {
    fn get_total_funds(&self) -> i64 {
        return self.available_funds + self.held_funds;
    }
}

struct DepositRecord {
    client: u16,
    amount: i64,
}

impl TryFrom<Transaction> for DepositRecord {
    type Error = &'static str;
    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        if let Transaction {
            r#type: DEPOSIT,
            client,
            amount,
        } = value
        {
            Ok(DepositRecord { client, amount })
        } else {
            return Err("Transaction is not a deposit");
        }
    }
}

#[derive(Default)]
pub struct BankDatabase {
    accounts: HashMap<u16, Account>,
    deposits: BTreeMap<u32, DepositRecord>,
    disputes: HashSet<u32>,
}

impl Display for BankDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client,available,held,total,locked\n")?;
        for (client, account) in self.accounts.iter() {
            write!(
                f,
                "{},{},{},{},{}\n",
                client,
                fixed_point_to_string(account.available_funds),
                fixed_point_to_string(account.held_funds),
                fixed_point_to_string(account.get_total_funds()),
                account.locked
            )?;
        }
        Ok(())
    }
}

impl BankDatabase {
    pub fn execute_transaction(&mut self, transaction_id_tuple: TransactionIdTuple) {
        let transaction_id = transaction_id_tuple.0;
        let transaction = transaction_id_tuple.1;

        match transaction.r#type {
            DEPOSIT => {
                let mut transaction_account = self.accounts.entry(transaction.client).or_default();
                transaction_account.available_funds += transaction.amount;
                self.deposits.insert(
                    transaction_id,
                    transaction
                        .try_into()
                        .expect("This only fails if transaction is not a deposit"),
                );
            }
            WITHDRAWAL => {
                let mut transaction_account = self.accounts.entry(transaction.client).or_default();
                if transaction_account.locked {
                    eprintln!("Error: Withdrawal attempted on frozen account");
                    return;
                } else if transaction_account.available_funds < transaction.amount {
                    eprintln!("Error: withdrawal bigger than available funds");
                    return;
                } else {
                    transaction_account.available_funds -= transaction.amount;
                }
            }
            DISPUTE => {
                if self.disputes.contains(&transaction_id) {
                    eprintln!("Warning: Dropped duplicate dispute claim");
                    return;
                }
                if let Some(disputed_deposit) = self.deposits.get(&transaction_id) {
                    let deposit_account = self
                        .accounts
                        .get_mut(&disputed_deposit.client)
                        .expect("Existence proven by deposit record");
                    deposit_account.available_funds -= disputed_deposit.amount;
                    deposit_account.held_funds += disputed_deposit.amount;
                    self.disputes.insert(transaction_id);
                } else {
                    eprintln!("Warning: Dropped dispute with invalid tx id")
                }
            }
            RESOLVE => {
                if !self.disputes.contains(&transaction_id) {
                    eprintln!("Warning: Dropped dispute resolve with undisputed tx");
                    return;
                }
                let disputed_deposit = self.deposits.get(&transaction_id)
                    .expect("That the dispute is contained in disputes implies that the deposit record exists");
                let deposit_account = self
                    .accounts
                    .get_mut(&disputed_deposit.client)
                    .expect("Existence proven by deposit record");
                deposit_account.held_funds -= disputed_deposit.amount;
                deposit_account.available_funds += disputed_deposit.amount;
                self.disputes.remove(&transaction_id);
            }
            CHARGEBACK => {
                if !self.disputes.contains(&transaction_id) {
                    eprintln!("Warning: Dropped chargeback with undisputed tx");
                    return;
                }
                let disputed_deposit = self.deposits.get(&transaction_id)
                    .expect("That the dispute is contained in disputes implies that the deposit record exists");
                let deposit_account = self
                    .accounts
                    .get_mut(&disputed_deposit.client)
                    .expect("Existence proven by deposit record");
                deposit_account.held_funds -= disputed_deposit.amount;
                deposit_account.locked = true;
                self.disputes.remove(&transaction_id);
            }
        }
        // Create account from transaction if it does not exist
        self.accounts.entry(transaction.client).or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT_1: u16 = 10;
    const CLIENT_DONT_CARE: u16 = 20;

    const GIVE_50_CLIENT_1: TransactionIdTuple = (
        1,
        Transaction {
            amount: 50,
            client: CLIENT_1,
            r#type: DEPOSIT,
        },
    );
    const GIVE_100_CLIENT_1: TransactionIdTuple = (
        2,
        Transaction {
            amount: 100,
            client: CLIENT_1,
            r#type: DEPOSIT,
        },
    );
    mod deposits {
        use super::*;

        #[test]
        fn deposit_works() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_100_CLIENT_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 100)
        }
    }

    const REMOVE_100_CLIENT_1: TransactionIdTuple = (
        3,
        Transaction {
            amount: 100,
            client: CLIENT_1,
            r#type: WITHDRAWAL,
        },
    );

    mod withdrawals {
        use super::*;
        #[test]
        fn withdrawal_works() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_100_CLIENT_1);
            bank.execute_transaction(GIVE_100_CLIENT_1);
            bank.execute_transaction(REMOVE_100_CLIENT_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 100)
        }

        #[test]
        fn withdrawal_rejected_insufficient_funds() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(REMOVE_100_CLIENT_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50)
        }
        #[test]
        fn withdrawal_rejected_account_locked() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(GIVE_100_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(CHARGEBACK_1);
            bank.execute_transaction(REMOVE_100_CLIENT_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 100);
        }
    }

    const DONT_CARE: i64 = 0;
    const INVALID_TRANSACTION: u32 = 999;

    const DISPUTE_1: TransactionIdTuple = (
        1,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: DISPUTE,
        },
    );
    const DISPUTE_INVALID: TransactionIdTuple = (
        INVALID_TRANSACTION,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: DISPUTE,
        },
    );
    const DISPUTE_NON_DEPOSIT: TransactionIdTuple = (
        3,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: DISPUTE,
        },
    );
    mod disputes {
        use super::*;

        #[test]
        fn dispute_works() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 50);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 0);
        }
        #[test]
        fn dispute_invalid_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_INVALID);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50);
        }
        #[test]
        fn dispute_non_deposit_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_100_CLIENT_1);
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(REMOVE_100_CLIENT_1);
            bank.execute_transaction(DISPUTE_NON_DEPOSIT);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50);
        }
        #[test]
        fn dispute_duplicate_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(DISPUTE_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 50);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 0);
        }
    }

    const RESOLVE_1: TransactionIdTuple = (
        1,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: RESOLVE,
        },
    );

    mod resolves {
        use super::*;

        #[test]
        fn resolve_works() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(RESOLVE_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50);
        }
        #[test]
        fn resolve_duplicate_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(RESOLVE_1);
            bank.execute_transaction(RESOLVE_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50);
        }
        #[test]
        fn resolve_invalid_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_100_CLIENT_1);
            bank.execute_transaction(RESOLVE_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 100);
        }
    }

    const CHARGEBACK_1: TransactionIdTuple = (
        1,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: CHARGEBACK,
        },
    );
    const CHARGEBACK_INVALID: TransactionIdTuple = (
        1,
        Transaction {
            amount: DONT_CARE,
            client: CLIENT_DONT_CARE,
            r#type: CHARGEBACK,
        },
    );

    mod chargebacks {
        use super::*;

        #[test]
        fn chargeback_works() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(CHARGEBACK_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().locked, true);
        }
        #[test]
        fn chargeback_duplicate_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(DISPUTE_1);
            bank.execute_transaction(CHARGEBACK_1);
            bank.execute_transaction(CHARGEBACK_1);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().locked, true);
        }
        #[test]
        fn chargeback_invalid_is_ignored() {
            let mut bank = BankDatabase::default();
            bank.execute_transaction(GIVE_50_CLIENT_1);
            bank.execute_transaction(CHARGEBACK_INVALID);

            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().held_funds, 0);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().available_funds, 50);
            assert_eq!(bank.accounts.get(&CLIENT_1).unwrap().locked, false);
        }
    }
}
