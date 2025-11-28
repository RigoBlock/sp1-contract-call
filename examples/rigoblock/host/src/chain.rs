use eyre::Result;
use sp1_cc_host_executor::Genesis;
use serde::{Deserialize, Serialize};

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
        }
    }

    /// Get the genesis configuration for this chain
    pub fn genesis(&self) -> Result<Genesis> {
        match self {
            SupportedChain::Ethereum => Ok(Genesis::Mainnet),
            //SupportedChain::Optimism => Ok(Genesis::OpMainnet),
            SupportedChain::Optimism => {
                let json = include_str!("../genesis/10.json");
                Ok(Genesis::Custom(serde_json::from_str(json)?))
            },
            SupportedChain::BnbChain => {
                let json = include_str!("../genesis/56.json");
                Ok(Genesis::Custom(serde_json::from_str(json)?))
            }
            SupportedChain::Base => {
                let json = include_str!("../genesis/8453.json");
                Ok(Genesis::Custom(serde_json::from_str(json)?))
            }
            SupportedChain::Arbitrum => {
                let json = include_str!("../genesis/42161.json");
                Ok(Genesis::Custom(serde_json::from_str(json)?))
            }
            SupportedChain::Unichain => {
                let json = include_str!("../genesis/130.json");
                Ok(Genesis::Custom(serde_json::from_str(json)?))
            }
        }
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
            1301 => Ok(SupportedChain::Unichain),
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
        assert_eq!(SupportedChain::Unichain.chain_id(), 1301);
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