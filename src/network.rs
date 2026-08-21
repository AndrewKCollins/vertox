use clap::ValueEnum;

pub const MAINNET_CHAIN_ID: u64 = 4663;
pub const TESTNET_CHAIN_ID: u64 = 46630;

pub const MAINNET_RPC: &str = "https://rpc.mainnet.chain.robinhood.com";
pub const TESTNET_RPC: &str = "https://rpc.testnet.chain.robinhood.com";

pub const MAINNET_EXPLORER: &str = "https://robinhoodchain.blockscout.com";
pub const TESTNET_EXPLORER: &str = "https://explorer.testnet.chain.robinhood.com";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn chain_id(self) -> u64 {
        match self {
            Self::Mainnet => MAINNET_CHAIN_ID,
            Self::Testnet => TESTNET_CHAIN_ID,
        }
    }

    pub fn default_rpc(self) -> &'static str {
        match self {
            Self::Mainnet => MAINNET_RPC,
            Self::Testnet => TESTNET_RPC,
        }
    }

    pub fn explorer(self) -> &'static str {
        match self {
            Self::Mainnet => MAINNET_EXPLORER,
            Self::Testnet => TESTNET_EXPLORER,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Mainnet => "Robinhood Chain",
            Self::Testnet => "Robinhood Chain Testnet",
        }
    }
}
