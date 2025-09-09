use ethers_contract_abigen::Abigen;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=abis/UniswapV2Router02.json");

    // Define the ABI content for a simplified Uniswap V2 router
    let uniswap_v2_abi_json = r#"[
        {
            "inputs": [
                { "internalType": "uint256", "name": "amountIn", "type": "uint256" },
                { "internalType": "address[]", "name": "path", "type": "address[]" }
            ],
            "name": "getAmountsOut",
            "outputs": [
                { "internalType": "uint256[]", "name": "amounts", "type": "uint256[]" }
            ],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#;

    // The Abigen builder
    let abigen = Abigen::new("UniswapRouter", uniswap_v2_abi_json).unwrap();

    // Generate the bindings and write them to a file
    let bindings = abigen.generate().unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut file = std::fs::File::create(Path::new(&out_dir).join("uniswap_v2_router.rs")).unwrap();
    file.write_all(bindings.to_string().as_bytes()).unwrap();
}