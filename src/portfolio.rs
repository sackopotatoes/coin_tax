use std::error::Error;
use std::collections::{HashMap};

use thiserror::Error;

use super::transaction;

#[derive(Error, Debug, PartialEq)]
pub enum PortfolioError {
  #[error("Error Accessing History for {attempted_access:?}")]
  HistoryAccessError {
    attempted_access: String
  }
}

type Portfolio = HashMap<String, AssetHistory>;

#[derive(Debug)]
pub(crate) struct AssetHistory {
  name: String,
  history: Vec<transaction::Transaction>,
  quantity: f32,
}

impl AssetHistory {
  fn push_into_history(&mut self, new_transaction: transaction::Transaction) {
    let pos = self.history.binary_search(&new_transaction).unwrap_or_else(|e| e);
    self.history.insert(pos, new_transaction);
  }

  fn add_transaction_to_asset(&mut self, new_transaction: transaction::Transaction) {
    match &new_transaction.action {
      transaction::TransactionType::Buy => {
        self.quantity += new_transaction.quantity;
      },
      transaction::TransactionType::Sell => {
        self.quantity -= new_transaction.quantity;
      },
      transaction::TransactionType::Income => {
        self.quantity += new_transaction.quantity;
      },
      transaction::TransactionType::Convert => {
        let conversion = new_transaction.conversion_to.clone().unwrap();
        
        if self.name == conversion.name {
          self.quantity += conversion.quantity;
        }
        else {
          self.quantity -= new_transaction.quantity;
        }
      }
    }

    self.push_into_history(new_transaction);
  }
}

fn add_new_asset_to_portfolio(mut portfolio: Portfolio, asset: &str) -> Portfolio {
  if !portfolio.contains_key(asset) {
    let asset_history = AssetHistory {
        name: String::from(asset),
        history: Vec::new(),
        quantity: 0.0
      };

    portfolio.insert(String::from(asset), asset_history);
  }
  
  portfolio
}


pub(crate) fn add_to_portfolio(mut portfolio: Portfolio, transaction: transaction::Transaction) -> Result<Portfolio, Box<dyn Error>> {
  portfolio = add_new_asset_to_portfolio(portfolio, &transaction.asset);

  // handle update to converted currency
  if transaction.action == transaction::TransactionType::Convert {
    let conversion = &transaction.conversion_to.clone().unwrap();

    portfolio = add_new_asset_to_portfolio(portfolio, &conversion.name);

    let coverted_to_asset = portfolio.get_mut(&conversion.name).ok_or(PortfolioError::HistoryAccessError{attempted_access: String::from(&conversion.name)})?;

    coverted_to_asset.add_transaction_to_asset(transaction.clone());
  }

  let asset_history = portfolio.get_mut(&transaction.asset).ok_or(PortfolioError::HistoryAccessError{attempted_access:String::from(&transaction.asset)})?;

  asset_history.add_transaction_to_asset(transaction);

  Ok(portfolio)
}