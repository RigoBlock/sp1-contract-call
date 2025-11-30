//use std::fs;

//use alloy_genesis::Genesis;
use eyre::Result;
//use reth_chainspec::ChainSpec;
use sp1_cc_host_executor::Genesis;
//use sp1_sdk::ChainSpec;
use serde::{Deserialize, Serialize};

pub const ETH_MAINNET_GENESIS_JSON: &str = include_str!("../genesis/1.json");
pub const UNICHAIN_GENESIS_JSON: &str = include_str!("../genesis/130.json");
pub const OPTIMISM_GENESIS_JSON: &str = include_str!("../genesis/10.json");
pub const BASE_GENESIS_JSON: &str = include_str!("../genesis/8453.json");
pub const ARBITRUM_GENESIS_JSON: &str = include_str!("../genesis/42161.json");
pub const BSC_GENESIS_JSON: &str = include_str!("../genesis/56.json");
pub const SEPOLIA_GENESIS_JSON: &str = include_str!("../genesis/11155111.json");

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportedChain {
    /// Ethereum Mainnet (Chain ID: 1)
    Ethereum,
    /// Optimism Mainnet (Chain ID: 10)
    Optimism,
    /// BNB Smart Chain (Chain ID: 56)
    BnbChain,
    /// Base (Chain ID: 8453)
    Base,
    /// Arbitrum One (Chain ID: 42161)
    Arbitrum,
    /// Unichain (Chain ID: 130)
    Unichain,
    Sepolia
}

impl SupportedChain {
    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        match self {
            SupportedChain::Ethereum => 1,
            SupportedChain::Optimism => 10,
            SupportedChain::BnbChain => 56,
            SupportedChain::Base => 8453,
            SupportedChain::Arbitrum => 42161,
            SupportedChain::Unichain => 130,
            SupportedChain::Sepolia => 11155111,
        }
    }

    /// Get the chain name as a string
    pub fn name(&self) -> &'static str {
        match self {
            SupportedChain::Ethereum => "ethereum",
            SupportedChain::Optimism => "optimism",
            SupportedChain::BnbChain => "bnb",
            SupportedChain::Base => "base",
            SupportedChain::Arbitrum => "arbitrum",
            SupportedChain::Unichain => "unichain",
            SupportedChain::Sepolia => "sepolia",
        }
    }


    //let genesis_json = fs::read_to_string(genesis_path)
    //    .map_err(|err| eyre::eyre!("Failed to read genesis file: {err}"))?;

    /// Get the genesis configuration for this chain
    pub fn genesis(&self) -> Result<Genesis> {
        let genesis_json = match self {
            SupportedChain::Ethereum => ETH_MAINNET_GENESIS_JSON,
            SupportedChain::Unichain => UNICHAIN_GENESIS_JSON,
            SupportedChain::Optimism => OPTIMISM_GENESIS_JSON,
            SupportedChain::BnbChain => BSC_GENESIS_JSON,
            SupportedChain::Base => BASE_GENESIS_JSON,
            SupportedChain::Arbitrum => ARBITRUM_GENESIS_JSON,
            SupportedChain::Sepolia => SEPOLIA_GENESIS_JSON,
            // add more as needed
        };

        let genesis = serde_json::from_str::<alloy_genesis::Genesis>(&genesis_json)?;

        Ok(Genesis::Custom(genesis.config))
    }

    /// Check if this is an L2 or altchain (not Ethereum mainnet)
    pub fn is_l2_or_altchain(&self) -> bool {
        !matches!(self, SupportedChain::Ethereum)
    }

    /// Check if this chain uses Optimism stack
    pub fn is_op_stack(&self) -> bool {
        matches!(
            self,
            SupportedChain::Optimism | SupportedChain::Base | SupportedChain::Unichain
        )
    }
}

impl std::str::FromStr for SupportedChain {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Ok(SupportedChain::Ethereum),
            "optimism" | "op" => Ok(SupportedChain::Optimism),
            "bnb" | "bsc" | "bnbchain" => Ok(SupportedChain::BnbChain),
            "base" => Ok(SupportedChain::Base),
            "arbitrum" | "arb" | "arbitrumone" => Ok(SupportedChain::Arbitrum),
            "unichain" | "uni" => Ok(SupportedChain::Unichain),
            "sepolia" => Ok(SupportedChain::Sepolia),
            _ => Err(eyre::eyre!("Unsupported chain: {}", s)),
        }
    }
}

impl std::fmt::Display for SupportedChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TryFrom<u64> for SupportedChain {
    type Error = eyre::Error;

    fn try_from(chain_id: u64) -> Result<Self> {
        match chain_id {
            1 => Ok(SupportedChain::Ethereum),
            10 => Ok(SupportedChain::Optimism),
            56 => Ok(SupportedChain::BnbChain),
            8453 => Ok(SupportedChain::Base),
            42161 => Ok(SupportedChain::Arbitrum),
            130 => Ok(SupportedChain::Unichain),
            11155111 => Ok(SupportedChain::Sepolia),
            _ => Err(eyre::eyre!("Unsupported chain ID: {}", chain_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_ids() {
        assert_eq!(SupportedChain::Ethereum.chain_id(), 1);
        assert_eq!(SupportedChain::Optimism.chain_id(), 10);
        assert_eq!(SupportedChain::BnbChain.chain_id(), 56);
        assert_eq!(SupportedChain::Base.chain_id(), 8453);
        assert_eq!(SupportedChain::Arbitrum.chain_id(), 42161);
        assert_eq!(SupportedChain::Unichain.chain_id(), 130);
        assert_eq!(SupportedChain::Sepolia.chain_id(), 11155111);
    }

    #[test]
    fn test_from_str() {
        assert_eq!("ethereum".parse::<SupportedChain>().unwrap(), SupportedChain::Ethereum);
        assert_eq!("optimism".parse::<SupportedChain>().unwrap(), SupportedChain::Optimism);
        assert_eq!("base".parse::<SupportedChain>().unwrap(), SupportedChain::Base);
    }

    #[test]
    fn test_is_l2() {
        assert!(!SupportedChain::Ethereum.is_l2_or_altchain());
        assert!(SupportedChain::Optimism.is_l2_or_altchain());
        assert!(SupportedChain::Base.is_l2_or_altchain());
    }

    #[test]
    fn test_is_op_stack() {
        assert!(!SupportedChain::Ethereum.is_op_stack());
        assert!(SupportedChain::Optimism.is_op_stack());
        assert!(SupportedChain::Base.is_op_stack());
        assert!(SupportedChain::Unichain.is_op_stack());
        assert!(!SupportedChain::Arbitrum.is_op_stack());
        assert!(!SupportedChain::BnbChain.is_op_stack());
    }
}