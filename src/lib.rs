use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::{HashMap};

use thiserror::Error;

mod transaction;
mod portfolio;

#[derive(Error, Debug, PartialEq)]
pub enum LibError {
  #[error("Error Accessing History")]
  HistoryAccessError
}

fn read_lines<P>(filename: P) -> io::Result<std::iter::Enumerate<io::Lines<io::BufReader<File>>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines().enumerate())
}

pub fn run(filename: &str, exchange: &str) -> Result<(), Box<dyn Error>> {
    let mut portfolio: HashMap<String, portfolio::AssetHistory> = HashMap::new();

    let lines = read_lines(filename)?;


    for (index, line) in lines {
        if index == 0 {
          //TODO: detect headers
          continue;
        }

        if let Ok(ip) = line {
          let transaction = transaction::create_transaction_from_line(&ip, &exchange)?;

          portfolio = portfolio::add_to_portfolio(portfolio, transaction)?;
        }
    }

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
