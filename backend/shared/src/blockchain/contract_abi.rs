//! Smart contract ABI definitions for Nexus Security

use ethers::abi::{Abi, Token};
use ethers::types::{H160, U256};
use serde_json::json;

/// Get the Payment Contract ABI
pub fn payment_contract_abi() -> Abi {
    serde_json::from_value(json!([
        {
            "inputs": [
                {"internalType": "bytes32", "name": "bountyId", "type": "bytes32"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "depositBounty",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "bytes32", "name": "bountyId", "type": "bytes32"},
                {"internalType": "address", "name": "winner", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "distributeBounty",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "bytes32", "name": "bountyId", "type": "bytes32"},
                {"internalType": "address", "name": "user", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "lockStake",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "bytes32", "name": "bountyId", "type": "bytes32"},
                {"internalType": "address", "name": "user", "type": "address"}
            ],
            "name": "unlockStake",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "bytes32", "name": "bountyId", "type": "bytes32"},
                {"internalType": "address", "name": "user", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "slashStake",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "user", "type": "address"}
            ],
            "name": "getBalance",
            "outputs": [
                {"internalType": "uint256", "name": "", "type": "uint256"}
            ],
            "stateMutability": "view",
            "type": "function"
        }
    ]))
    .expect("Valid ABI")
}

/// Get the ERC20 Token ABI (for reward token)
pub fn erc20_abi() -> Abi {
    serde_json::from_value(json!([
        {
            "inputs": [
                {"internalType": "address", "name": "spender", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "approve",
            "outputs": [
                {"internalType": "bool", "name": "", "type": "bool"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "recipient", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "transfer",
            "outputs": [
                {"internalType": "bool", "name": "", "type": "bool"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "sender", "type": "address"},
                {"internalType": "address", "name": "recipient", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "transferFrom",
            "outputs": [
                {"internalType": "bool", "name": "", "type": "bool"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "account", "type": "address"}
            ],
            "name": "balanceOf",
            "outputs": [
                {"internalType": "uint256", "name": "", "type": "uint256"}
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [
                {"internalType": "uint256", "name": "", "type": "uint256"}
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "from", "type": "address"},
                {"indexed": true, "internalType": "address", "name": "to", "type": "address"},
                {"indexed": false, "internalType": "uint256", "name": "value", "type": "uint256"}
            ],
            "name": "Transfer",
            "type": "event"
        }
    ]))
    .expect("Valid ABI")
}

/// Encode function call data
pub fn encode_deposit_bounty(bounty_id: [u8; 32], amount: U256) -> Vec<u8> {
    let function = payment_contract_abi()
        .function("depositBounty")
        .expect("Function exists");
    
    function
        .encode_input(&[Token::FixedBytes(bounty_id.to_vec()), Token::Uint(amount)])
        .expect("Valid encoding")
}

/// Encode stake lock call
pub fn encode_lock_stake(bounty_id: [u8; 32], user: H160, amount: U256) -> Vec<u8> {
    let function = payment_contract_abi()
        .function("lockStake")
        .expect("Function exists");
    
    function
        .encode_input(&[
            Token::FixedBytes(bounty_id.to_vec()),
            Token::Address(user),
            Token::Uint(amount),
        ])
        .expect("Valid encoding")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_contract_abi() {
        let abi = payment_contract_abi();
        assert!(abi.function("depositBounty").is_ok());
        assert!(abi.function("distributeBounty").is_ok());
        assert!(abi.function("lockStake").is_ok());
    }

    #[test]
    fn test_erc20_abi() {
        let abi = erc20_abi();
        assert!(abi.function("transfer").is_ok());
        assert!(abi.function("balanceOf").is_ok());
        assert!(abi.function("approve").is_ok());
    }
}
