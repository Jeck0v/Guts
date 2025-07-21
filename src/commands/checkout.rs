use anyhow::Result;
use clap::Args;


#[derive(Args)]
pub struct CheckoutObject {
    pub name : String
}

pub fn run(args:CheckoutObject) -> Result<String> {
    
}