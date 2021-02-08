use std::error::Error;
use std::collections::{HashMap};

use thiserror::Error;

use super::transaction;

#[derive(Error, Debug, PartialEq)]
pub enum PortfolioError {
  #[error("Error Accessing History")]
  HistoryAccessError
}

#[derive(Debug)]
pub(crate) struct AssetHistory {
  name: String,
  history: Vec<transaction::Transaction>,
  quantity: f64,
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
        self.quantity -= new_transaction.quantity;
      }
    }

    self.push_into_history(new_transaction);
  }
}



pub(crate) fn add_to_portfolio(mut portfolio: HashMap<String, AssetHistory>, transaction: transaction::Transaction) -> Result<HashMap<String, AssetHistory>, Box<dyn Error>> {
  if !portfolio.contains_key(&transaction.asset) {
    let name = transaction.asset.clone();
    let asset_history = AssetHistory {
      name,
      history: Vec::new(),
      quantity: transaction.quantity
    };

    portfolio.insert(transaction.asset.clone(), asset_history);

    portfolio.get_mut(&transaction.asset).ok_or(PortfolioError::HistoryAccessError)?.history.push(transaction.clone());
  }
  else {
    let asset_history = portfolio.get_mut(&transaction.asset).ok_or(PortfolioError::HistoryAccessError)?;

    asset_history.add_transaction_to_asset(transaction);
  }

  Ok(portfolio)
}