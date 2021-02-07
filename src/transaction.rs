use std::error::Error;
use std::str::FromStr;

use chrono::{DateTime};

#[derive(Debug)]
#[derive(PartialEq)]
pub(crate) enum TransactionType {
  Buy,
  Sell,
  Income,
  Convert
}

#[derive(Debug)]
pub(crate) struct Transaction {
  pub(crate) timestamp: i64,
  pub(crate) action: TransactionType,
  pub(crate) asset: String,
  pub(crate) quantity: f64,
  pub(crate) price: f64
}

fn get_coinbase_action(raw_action: &str) -> Result<TransactionType, &'static str> {
  match raw_action {
    "Buy" => Ok(TransactionType::Buy),
    "Sell" => Ok(TransactionType::Sell),
    "Rewards Income" => Ok(TransactionType::Income),
    "Coinbase Earn" => Ok(TransactionType::Income),
    "Convert" => Ok(TransactionType::Convert),
    _ => Err("Unknown Action!")
  }
}

fn get_action_by_exchange(raw_action: &str, exchange: &str) -> Result<TransactionType, &'static str> {
  match exchange {
    "coinbase" => Ok(get_coinbase_action(raw_action)?),
    _ => Err("Exchange not yet supported!")
  }
}

fn split_string(string: &str, delimeter: Option<char>) -> Vec<&str> {
  string.split(delimeter.unwrap_or(',')).collect()
}

fn split_csv_line(line: &str) -> Vec<&str> {
  split_string(&line, None)
}

pub(crate) fn create_transaction_from_line(line: &str) -> Result<Transaction, Box<dyn Error>> {
  let split_line = split_csv_line(line);

  //TODO: allow the indices to be keyed off exchange type
  let timestamp = DateTime::parse_from_rfc3339(split_line[0])?.timestamp_millis();
  let action = get_action_by_exchange(split_line[1], "coinbase")?;
  let asset = String::from(split_line[2]);
  let quantity = f64::from_str(split_line[3])?;
  let price = f64::from_str(split_line[6])?;

  Ok(Transaction {
    timestamp,
    action,
    asset,
    quantity,
    price
  })
}

#[cfg(test)]
mod tests {
  use super::*;

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
  }

  #[test]
  fn test_split_string() {
      let expected_result = vec!["this", "is", "a", "test", "string"];
      let test_string = "this,is,a,test,string";
      let pipe_separated = "this|is|a|test|string";

      let my_split_string = split_string(&test_string, None);
      let pipe_split_string = split_string(&pipe_separated, Some('|'));

      assert_eq!(my_split_string.len(), expected_result.len());
      assert!(do_vecs_match(&my_split_string, &expected_result));
      assert_eq!(pipe_split_string.len(), expected_result.len());
      assert!(do_vecs_match(&pipe_split_string, &expected_result));
  }

  #[test]
  fn test_split_csv_line() {
    let expected_result = vec!["this", "is", "a", "test", "string"];
    let test_string = "this,is,a,test,string";
    let pipe_separated = "this|is|a|test|string";

    let my_split_string = split_csv_line(&test_string);
    let pipe_split_string = split_csv_line(&pipe_separated);

    assert_eq!(my_split_string.len(), expected_result.len());
    assert!(do_vecs_match(&my_split_string, &expected_result));
    assert_ne!(pipe_split_string.len(), expected_result.len());
    assert!(!do_vecs_match(&pipe_split_string, &expected_result));
  }

  #[test]
  fn test_create_transaction_from_line() {
    let test_line = "2018-01-23T03:40:11Z,Buy,BTC,0.000919,10881.58,10.00,10.00,0.00,Bought 0.000919 BTC for $10.00 USD";

    let expected_result = Transaction {
      timestamp: 1516678811000,
      action: TransactionType::Buy,
      asset: String::from("BTC"),
      quantity: 0.000919,
      price: 10.00
    };

    let transaction = create_transaction_from_line(&test_line).unwrap();

    assert_eq!(transaction.timestamp, expected_result.timestamp);
    assert_eq!(transaction.action, expected_result.action);
    assert_eq!(transaction.asset, expected_result.asset);
    assert_eq!(transaction.timestamp, expected_result.timestamp);
    assert_eq!(transaction.timestamp, expected_result.timestamp);
  }
}