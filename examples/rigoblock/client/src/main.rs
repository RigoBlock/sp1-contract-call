#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{Address, Bytes, hex};
use sp1_cc_client_executor::{io::EvmSketchInput, ClientExecutor, ContractInput , Genesis};

/// Determine if a genesis configuration represents an OP stack chain
fn is_op_stack_chain(genesis: &Genesis) -> bool {
    match genesis {
        Genesis::OpMainnet => true,
        Genesis::Custom(config) => {
            // Base (8453), Unichain (130), and Optimism (10) use OP stack
            // Chain 10 will use Genesis::OpMainnet, but if custom genesis is used for any reason
            matches!(config.chain_id, 10 | 8453 | 130)
        }
        _ => false,
    }
}

pub fn main() {
    // Read the state sketch from stdin. Use this during the execution in order to
    // access Ethereum state.
    let state_sketch_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let state_sketch = bincode::deserialize::<EvmSketchInput>(&state_sketch_bytes).unwrap();

    // Read the vault address from stdin
    let vault_address = sp1_zkvm::io::read::<Address>();

    // Prepare bytecode for wrapper contract deployment (returns 64 bytes: value + timestamp)
    const BYTECODE: &str = "608060405234801561000f575f5ffd5b5060405161017738038061017783398101604081905261002e916100ef565b806001600160a01b031663e7d8724e6040518163ffffffff1660e01b81526004015f604051808303815f87803b158015610066575f5ffd5b505af1158015610078573d5f5f3e3d5ffd5b505050505f816001600160a01b03166389c065686040518163ffffffff1660e01b81526004016040805180830381865afa1580156100b8573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906100dc919061011c565b80515f8181524260205291925090604090f35b5f602082840312156100ff575f5ffd5b81516001600160a01b0381168114610115575f5ffd5b9392505050565b5f604082840312801561012d575f5ffd5b50604080519081016001600160401b038111828210171561015c57634e487b7160e01b5f52604160045260245ffd5b60405282518152602092830151928101929092525091905056fe";
    let bytecode_bytes = hex::decode(BYTECODE).expect("Decoding failed");
    let mut create_calldata = Vec::<u8>::from(bytecode_bytes);
    let constructor_args = alloy_sol_types::SolValue::abi_encode(&vault_address);
    create_calldata.extend_from_slice(&constructor_args);
    let create_calldata: Bytes = Bytes::from(create_calldata);

    let create_input = ContractInput::new_create(Address::default(), create_calldata);

    // Execute based on chain type
    // For OP stack chains (Optimism, Base, Unichain), use the optimism executor
    // For other chains (Ethereum, Arbitrum, BNB Chain), use the eth executor
    if is_op_stack_chain(&state_sketch.genesis) {
        let executor = ClientExecutor::optimism(&state_sketch).unwrap();
        executor.execute(create_input).unwrap();
    } else {
        let executor = ClientExecutor::eth(&state_sketch).unwrap();
        executor.execute(create_input).unwrap();
    }
}