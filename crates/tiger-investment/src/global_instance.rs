use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::manager::InvestmentManager;

lazy_static! {
    static ref INVESTMENT_MANAGER: RwLock<Option<Arc<InvestmentManager>>> = RwLock::new(None);
}

pub fn initialize_investment_manager(im: &Arc<InvestmentManager>) {
    *INVESTMENT_MANAGER.write() = Some(im.clone());
}

pub fn finalize_investment_manager() {
    *INVESTMENT_MANAGER.write() = None;
}

pub fn investment_manager_checked() -> anyhow::Result<Arc<InvestmentManager>> {
    INVESTMENT_MANAGER
        .read()
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Investment manager is not initialized!"))
}

pub fn investment_manager() -> Arc<InvestmentManager> {
    investment_manager_checked().unwrap()
}
