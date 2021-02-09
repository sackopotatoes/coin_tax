use std::error::Error;
use std::str::FromStr;
use std::cmp::{PartialEq, PartialOrd, Ord, Ordering};

use chrono::{DateTime};
use thiserror::Error;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub(crate) enum TransactionType {
  Buy,
  Sell,
  Income,
  Convert
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CoinConversion {
  pub (crate) name: String,
  pub (crate) quantity: f32
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Transaction {
  pub(crate) timestamp: i64,
  pub(crate) action: TransactionType,
  pub(crate) asset: String,
  pub(crate) quantity: f32,
  pub(crate) price: f32,
  pub(crate) conversion_to: Option<CoinConversion>
}

impl Eq for Transaction {}

impl PartialOrd for Transaction {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.timestamp.partial_cmp(&other.timestamp)
  }
}

impl Ord for Transaction {
  fn cmp(&self, other: &Self) -> Ordering {
    self.timestamp.cmp(&other.timestamp)
  }
}

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
  #[error("Unknown Action")]
  UnknownAction,
  #[error("Exchange not yet supported!")]
  UnsupportedExchange,
  #[error(transparent)]
  TimeParseError(#[from] chrono::ParseError),
  #[error(transparent)]
  FloatParseError(#[from] std::num::ParseFloatError)
}

fn get_coinbase_action(raw_action: &str) -> Result<TransactionType, TransactionError> {
  match raw_action {
    "Buy" => Ok(TransactionType::Buy),
    "Sell" => Ok(TransactionType::Sell),
    "Rewards Income" => Ok(TransactionType::Income),
    "Coinbase Earn" => Ok(TransactionType::Income),
    "Convert" => Ok(TransactionType::Convert),
    _ => Err(TransactionError::UnknownAction)
  }
}

fn get_action_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<TransactionType, TransactionError> {
  match exchange {
    "coinbase" => Ok(get_coinbase_action(line_data[1])?),
    _ => Err(TransactionError::UnsupportedExchange)
  }
}

fn get_timestamp_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<i64, TransactionError> {
  match exchange {
    "coinbase" => Ok(DateTime::parse_from_rfc3339(line_data[0])?.timestamp_millis()),
    _ => Err(TransactionError::UnsupportedExchange)
  }
}

fn get_asset_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<String, TransactionError> {
  match exchange {
    "coinbase" => Ok(String::from(line_data[2])),
    _ => Err(TransactionError::UnsupportedExchange)
  }
}

fn get_quantity_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<f32, TransactionError> {
  match exchange {
    "coinbase" => Ok(f32::from_str(line_data[3])?),
    _ => Err(TransactionError::UnsupportedExchange)
  }
}

fn get_price_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<f32, TransactionError> {
    match exchange {
    "coinbase" => Ok(f32::from_str(line_data[6])?),
    _ => Err(TransactionError::UnsupportedExchange)
  }
}

fn get_conversion_to_by_exchange(line_data: &Vec<&str>, exchange: &str) -> Result<Option<CoinConversion>, TransactionError> {
    match exchange {
    "coinbase" => {
      let note_data: Vec<&str> = line_data.last().unwrap().split(" ").collect::<Vec<&str>>();

      Ok(Some(CoinConversion {
        name: String::from(*note_data.last().unwrap()).replace('"', ""),
        quantity: f32::from_str(note_data[note_data.len() - 2])?
      }))
    },
    _ => Err(TransactionError::UnsupportedExchange)
  }
}



fn split_string(string: &str, delimeter: Option<char>) -> Vec<&str> {
  string.split(delimeter.unwrap_or(',')).collect()
}

fn split_csv_line(line: &str) -> Vec<&str> {
  split_string(&line, None)
}

pub(crate) fn create_transaction_from_line(line: &str, exchange: &str) -> Result<Transaction, Box<dyn Error>> {
  let split_line = split_csv_line(line);

  let timestamp = get_timestamp_by_exchange(&split_line, &exchange)?;
  let action = get_action_by_exchange(&split_line, &exchange)?;
  let asset = get_asset_by_exchange(&split_line, &exchange)?;
  let quantity = get_quantity_by_exchange(&split_line, &exchange)?;
  let price = get_price_by_exchange(&split_line, &exchange)?;
  let mut conversion_to = None;

  if action == TransactionType::Convert {
    conversion_to = get_conversion_to_by_exchange(&split_line, &exchange)?;
  }

  Ok(Transaction {
    timestamp,
    action,
    asset,
    quantity,
    price,
    conversion_to
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
  fn test_get_action_by_exchange() -> Result<(), Box<dyn Error>> {
    let test_buy_data = vec!["2018-01-23T03:40:11Z","Buy","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let test_sell_data = vec!["2018-01-23T03:40:11Z","Sell","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let test_income_data = vec!["2018-01-23T03:40:11Z","Rewards Income","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let test_earn_data = vec!["2018-01-23T03:40:11Z","Coinbase Earn","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let test_convert_data = vec!["2018-01-23T03:40:11Z","Convert","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let test_unknown_data = vec!["2018-01-23T03:40:11Z","Unknown","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];

    let my_buy_action = get_action_by_exchange(&test_buy_data, "coinbase")?;
    let my_sell_action = get_action_by_exchange(&test_sell_data, "coinbase")?;
    let my_income_action = get_action_by_exchange(&test_income_data, "coinbase")?;
    let my_earn_action = get_action_by_exchange(&test_earn_data, "coinbase")?;
    let my_convert_action = get_action_by_exchange(&test_convert_data, "coinbase")?;
    let my_unknown_action = get_action_by_exchange(&test_unknown_data, "coinbase").unwrap_err();
    let my_unsupported_exchange = get_action_by_exchange(&test_buy_data, "coinfake").unwrap_err();

    assert_eq!(my_buy_action, TransactionType::Buy);
    assert_eq!(my_sell_action, TransactionType::Sell);
    assert_eq!(my_income_action, TransactionType::Income);
    assert_eq!(my_earn_action, TransactionType::Income);
    assert_eq!(my_convert_action, TransactionType::Convert);
    assert_eq!(my_unknown_action, TransactionError::UnknownAction);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);

    Ok(())
  }

  #[test]
  fn test_get_timestamp_by_exchange() {
    let test_data = vec!["2018-01-23T03:40:11Z","Buy","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];

    let expected_result = 1516678811000;

    let my_timestamp = get_timestamp_by_exchange(&test_data, "coinbase").unwrap();
    let my_unsupported_exchange = get_timestamp_by_exchange(&test_data, "coinfake").unwrap_err();

    assert_eq!(my_timestamp, expected_result);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);
  }

  #[test]
  fn test_get_asset_by_exchange() {
    let test_data = vec!["2018-01-23T03:40:11Z","Buy","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let expected_result = "BTC";

    let my_asset = get_asset_by_exchange(&test_data, "coinbase").unwrap();
    let my_unsupported_exchange = get_asset_by_exchange(&test_data, "coinfake").unwrap_err();

    assert_eq!(my_asset, expected_result);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);
  }

  #[test]
  fn test_get_quantity_by_exchange() {
    let test_data = vec!["2018-01-23T03:40:11Z","Buy","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let expected_result = 0.000919;

    let my_quantity = get_quantity_by_exchange(&test_data, "coinbase").unwrap();
    let my_unsupported_exchange = get_quantity_by_exchange(&test_data, "coinfake").unwrap_err();

    assert_eq!(my_quantity, expected_result);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);
  }

  #[test]
  fn test_get_price_by_exchange() {
    let test_data = vec!["2018-01-23T03:40:11Z","Buy","BTC","0.000919","10881.58","10.00","10.00","0.00","Bought 0.000919 BTC for $10.00 USD"];
    let expected_result = 10.00;

    let my_price = get_price_by_exchange(&test_data, "coinbase").unwrap();
    let my_unsupported_exchange = get_price_by_exchange(&test_data, "coinfake").unwrap_err();

    assert_eq!(my_price, expected_result);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);
  }

  #[test]
  fn test_get_conversion_to_by_exchange() {
    let test_data = vec!["2021-01-31T05:20:47Z","Convert","XLM","1641.4065951","0.310000","505.34","515.01","9.67","Converted 1,641.4065951 XLM to 774.762752 ALGO"];
    let expected_result = Some(CoinConversion {
      name: String::from("ALGO"),
      quantity: 774.762752
    });

    let my_conversion_to = get_conversion_to_by_exchange(&test_data, "coinbase").unwrap();
    let my_unsupported_exchange = get_price_by_exchange(&test_data, "coinfake").unwrap_err();

    assert_eq!(my_conversion_to, expected_result);
    assert_eq!(my_unsupported_exchange, TransactionError::UnsupportedExchange);
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
    let test_string = "2018-01-23T03:40:11Z,Buy,BTC,0.000919,10881.58,10.00,10.00,0.00,Bought 0.000919 BTC for $10.00 USD";
    let expected_result = Transaction {
      timestamp: 1516678811000,
      action: TransactionType::Buy,
      asset: String::from("BTC"),
      quantity: 0.000919,
      price: 10.00,
      conversion_to: None
    };

    let transaction = create_transaction_from_line(&test_string, "coinbase").unwrap();

    assert_eq!(transaction.timestamp, expected_result.timestamp);
    assert_eq!(transaction.action, expected_result.action);
    assert_eq!(transaction.asset, expected_result.asset);
    assert_eq!(transaction.timestamp, expected_result.timestamp);
    assert_eq!(transaction.timestamp, expected_result.timestamp);
  }
}