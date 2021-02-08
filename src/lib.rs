use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::{HashMap};

use thiserror::Error;

mod transaction;

#[derive(Error, Debug, PartialEq)]
pub enum LibError {
  #[error("Error Accessing History")]
  HistoryAccessError
}

//TODO: build coin buckets
#[derive(Debug)]
struct AssetHistory {
  name: String,
  history: Vec<transaction::Transaction>,
  quantity: f64,
}

fn read_lines<P>(filename: P) -> io::Result<std::iter::Enumerate<io::Lines<io::BufReader<File>>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines().enumerate())
}

pub fn run(filename: &str, exchange: &str) -> Result<(), Box<dyn Error>> {
    let mut transactions: Vec<transaction::Transaction> = Vec::new();
    let mut portfolio: HashMap<String, AssetHistory> = HashMap::new();

    let lines = read_lines(filename)?;


    for (index, line) in lines {
        if index == 0 {
          //TODO: detect headers
          continue;
        }

        if let Ok(ip) = line {
          let transaction = transaction::create_transaction_from_line(&ip, &exchange)?;

          if !portfolio.contains_key(&transaction.asset) {
            let name = transaction.asset.clone();
            let asset_history = AssetHistory {
              name,
              history: Vec::new(),
              quantity: transaction.quantity
            };

            portfolio.insert(transaction.asset.clone(), asset_history);

            portfolio.get_mut(&transaction.asset).ok_or(LibError::HistoryAccessError)?.history.push(transaction.clone());
          }
          else {
            let mut asset_history = portfolio.get_mut(&transaction.asset).ok_or(LibError::HistoryAccessError)?;

            match &transaction.action {
              transaction::TransactionType::Buy => {
                asset_history.quantity += transaction.quantity;
              },
              transaction::TransactionType::Sell => {
                asset_history.quantity -= transaction.quantity;
              },
              transaction::TransactionType::Income => {
                asset_history.quantity += transaction.quantity;
              },
              transaction::TransactionType::Convert => {
                asset_history.quantity -= transaction.quantity;
              }
            }

            asset_history.history.push(transaction.clone());
          }

          transactions.push(transaction);
        }
    }

    // sort transactions by timestamp oldest -> newest
    transactions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    println!("{:#?}", portfolio);

    //TODO: start to build coin buckets to calc cost-basis

    Ok(())
}

#[cfg(test)]
mod lib_tests {
  use super::*;

  #[test]
  fn test_coinbase_run() -> Result<(), Box<dyn Error>> {
    match run("coinbase_test.csv", "coinbase") {
      Ok(_) => Ok(()),
      Err(e) => Err(e)
    }
  }
}
