//use std::path::PathBuf;

use alloy::hex;
use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_types::BlockNumberOrTag;
use alloy_sol_types::{SolValue};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::ContractPublicValues;
use sp1_cc_host_executor::EvmSketch;
use sp1_sdk::{include_elf, utils, /*HashableKey,*/ ProverClient, /*SP1ProofWithPublicValues,*/ SP1Stdin};
use url::Url;

use rigoblock_host::SupportedChain;

/// The bytecode of the wrapper contract (returns 64 bytes: value + timestamp).
const BYTECODE: &str = "608060405234801561000f575f5ffd5b5060405161017738038061017783398101604081905261002e916100ef565b806001600160a01b031663e7d8724e6040518163ffffffff1660e01b81526004015f604051808303815f87803b158015610066575f5ffd5b505af1158015610078573d5f5f3e3d5ffd5b505050505f816001600160a01b03166389c065686040518163ffffffff1660e01b81526004016040805180830381865afa1580156100b8573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906100dc919061011c565b80515f8181524260205291925090604090f35b5f602082840312156100ff575f5ffd5b81516001600160a01b0381168114610115575f5ffd5b9392505050565b5f604082840312801561012d575f5ffd5b50604080519081016001600160401b038111828210171561015c57634e487b7160e01b5f52604160045260245ffd5b60405282518152602092830151928101929092525091905056fe";

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_elf!("rigoblock-client");

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1CCProofFixture {
    vkey: String,
    public_values: String,
    proof: String,
}

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Generate a proof (expensive operation, may take several minutes)
    #[clap(long)]
    prove: bool,

    /// Chain name (ethereum, optimism, base, arbitrum, bnb, unichain)
    #[clap(long, env)]
    chain: String,

    /// RigoBlock vault address
    #[clap(long, env)]
    vault_address: Address,
}

/// Generate a `SP1CCProofFixture`, and save it as a json file.
///
/// This is useful for verifying the proof of contract call execution on chain.
/*fn save_fixture(vkey: String, proof: &SP1ProofWithPublicValues) -> eyre::Result<()> {
    let fixture = SP1CCProofFixture {
        vkey,
        public_values: format!("0x{}", hex::encode(proof.public_values.as_slice())),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    std::fs::create_dir_all(&fixture_path)?;
    std::fs::write(
        fixture_path.join("plonk-fixture.json"),
        serde_json::to_string_pretty(&fixture)?,
    )?;
    Ok(())
}*/

/// Get RPC URL for a given chain from environment variables.
fn get_rpc_url_for_chain(chain: &SupportedChain) -> eyre::Result<Url> {
    let env_var = format!("{}_RPC_URL", chain.name().to_uppercase());
    let rpc_url = std::env::var(&env_var)
        .map_err(|_| eyre::eyre!(
            "RPC URL not found for chain {}. Please set {} in your .env file", 
            chain.name(), 
            env_var
        ))?;
    
    Url::parse(&rpc_url)
        .map_err(|e| eyre::eyre!("Invalid RPC URL for chain {}: {}", chain.name(), e))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv().ok();

    // Setup logging.
    utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    // Parse the chain
    let chain: SupportedChain = args.chain.parse()?;

    println!("RigoBlock Unitary Value ZK Proof Generator");
    println!("==========================================");
    println!("Chain: {} (Chain ID: {})", chain.name(), chain.chain_id());
    println!("Vault Address: {}", args.vault_address);

    // Get RPC URL for the chain
    let eth_rpc_url = get_rpc_url_for_chain(&chain)?;
    println!("RPC URL: {}", eth_rpc_url);
    println!();

    // Prepare the host executor.
    println!("Setting up EVM state sketch...");

    // Execute contract deployment
    let bytecode_bytes = hex::decode(BYTECODE).expect("Decoding failed");
    let mut create_calldata = Vec::<u8>::from(bytecode_bytes);
    let constructor_args = alloy_sol_types::SolValue::abi_encode(&args.vault_address);
    create_calldata.extend_from_slice(&constructor_args);
    let create_calldata: Bytes = Bytes::from(create_calldata);

    // Handle execution based on chain type to avoid type mismatch
    // EthPrimitives chains vs OpPrimitives chains need separate handling
    let is_op_stack = matches!(chain, SupportedChain::Optimism | SupportedChain::Base | SupportedChain::Unichain);
    
    let input;
    if is_op_stack {
        // OP stack chains use OpPrimitives
        let sketch = match chain {
            SupportedChain::Optimism => {
                EvmSketch::builder()
                    //.at_block(BlockNumberOrTag::Latest)
                    .optimism_mainnet()
                    .el_rpc_url(eth_rpc_url)
                    .build()
                    .await?
            }
            SupportedChain::Base | SupportedChain::Unichain => {
                EvmSketch::builder()
                    //.at_block(BlockNumberOrTag::Latest)
                    .with_genesis(chain.genesis()?)
                    .el_rpc_url(eth_rpc_url)
                    .optimism()
                    .build()
                    .await?
            }
            _ => unreachable!()
        };

        println!("Checking call-wrapper contract deployment...");
        let check_nav = sketch.create(Address::default(), create_calldata).await?;
        
        // Decode 64 bytes: value (32 bytes) + timestamp (32 bytes)
        if check_nav.len() != 64 {
            return Err(eyre::eyre!("Expected 64 bytes from wrapper contract, got {}", check_nav.len()));
        }
        let decoded_value: U256 = U256::abi_decode(&check_nav[0..32])?;
        let decoded_timestamp: U256 = U256::abi_decode(&check_nav[32..64])?;

        println!();
        println!("Vault State:");
        println!("  Unitary Value: {}", decoded_value);
        println!("  Timestamp: {}", decoded_timestamp);
        println!();
        
        let block_hash = sketch.anchor.resolve().hash;
        println!("Block hash: {:#x}", block_hash);

        println!("Finalizing state sketch...");
        input = sketch.finalize().await?;
    } else {
        // Non-OP stack chains use EthPrimitives
        let sketch = match chain {
            SupportedChain::Ethereum | SupportedChain::Sepolia => {
                EvmSketch::builder()
                    .at_block(BlockNumberOrTag::Latest)
                    .with_genesis(chain.genesis()?)
                    .el_rpc_url(eth_rpc_url)
                    .build()
                    .await?
            }
            SupportedChain::Arbitrum | SupportedChain::BnbChain => {
                EvmSketch::builder()
                    .at_block(BlockNumberOrTag::Latest)
                    .with_genesis(chain.genesis()?)
                    .el_rpc_url(eth_rpc_url)
                    .build()
                    .await?
            }
            _ => unreachable!()
        };

        println!("Checking call-wrapper contract deployment...");
        let check_nav = sketch.create(Address::default(), create_calldata.clone()).await?;
        
        // Decode 64 bytes: value (32 bytes) + timestamp (32 bytes)
        if check_nav.len() != 64 {
            return Err(eyre::eyre!("Expected 64 bytes from wrapper contract, got {}", check_nav.len()));
        }
        let decoded_value: U256 = U256::abi_decode(&check_nav[0..32])?;
        let decoded_timestamp: U256 = U256::abi_decode(&check_nav[32..64])?;

        println!();
        println!("Vault State:");
        println!("  Unitary Value: {}", decoded_value);
        println!("  Timestamp: {}", decoded_timestamp);
        println!();
        
        // Keep track of the block hash for validation
        let block_hash = sketch.anchor.resolve().hash;
        println!("Block hash: {:#x}", block_hash);

        // Now that we've executed all of the calls, get the `EVMStateSketch` from the host executor.
        println!("Finalizing state sketch...");
        input = sketch.finalize().await?;
    }

    let input_bytes = bincode::serialize(&input)?;
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_bytes);
    stdin.write(&args.vault_address);
    
    let client = ProverClient::from_env();

    println!("Executing program in zkVM...");
    let (_, report) = client.execute(ELF, &stdin).run().map_err(|e| eyre::eyre!(e))?;
    println!(
        "✓ Program executed successfully with {} cycles",
        report.total_instruction_count()
    );
    println!();

    if !args.prove {
        println!("Skipping proof generation (use --prove flag to generate proof)");
        return Ok(());
    }

    println!("Generating ZK proof (this may take several minutes)...");
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, &stdin).plonk().run().map_err(|e| eyre::eyre!(e))?;
    println!("✓ Proof generated successfully");
    println!();

    let public_vals = ContractPublicValues::abi_decode(proof.public_values.as_slice())?;

    /*if public_vals.anchorHash != block_hash {
        return Err(eyre::eyre!(
            "Block hash mismatch! Expected {:#x}, got {:#x}",
            block_hash,
            public_vals.anchorHash
        ));
    }
    println!("✓ Block hash verified: {:#x}", block_hash);*/

    // Decode 64 bytes from proof: value (32 bytes) + timestamp (32 bytes)
    if public_vals.contractOutput.len() != 64 {
        return Err(eyre::eyre!(
            "Expected 64 bytes in proof output, got {}", 
            public_vals.contractOutput.len()
        ));
    }
    let unitary_value_from_proof: U256 = U256::abi_decode(&public_vals.contractOutput[0..32])?;
    let timestamp_from_proof: U256 = U256::abi_decode(&public_vals.contractOutput[32..64])?;

    println!();
    println!("Proven Vault State:");
    println!("  Unitary Value: {}", unitary_value_from_proof);
    println!("  Timestamp: {}", timestamp_from_proof);
    println!();

    //save_fixture(vk.bytes32(), &proof)?;
    //println!("✓ Proof saved to host/fixtures/plonk-fixture.json");

    client.verify(&proof, &vk)?;
    println!("✓ Proof verified successfully");
    println!();
    println!("==========================================");
    println!("Success! ZK proof generation complete.");
    return Ok(());
}