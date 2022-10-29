use crate::contract_vm::rpc_mock::{
    Bank, CwRpcClient, RpcContractInstance, RpcMockApi, RpcMockQuerier, RpcMockStorage,
};
use crate::contract_vm::Error;

use cosmwasm_std::{Addr, BlockInfo, Coin, ContractInfo, Env, Timestamp, TransactionInfo};
use cosmwasm_vm::{Backend, InstanceOptions, Storage};
use std::collections::HashMap;

type RpcBackend = Backend<RpcMockApi, RpcMockStorage, RpcMockQuerier>;

pub struct Model {
    instances: HashMap<String, RpcContractInstance>,
    bank: Bank,
    client: CwRpcClient,
    // similar to tx.origin of solidity
    eoa: String,

    // fields related to blockchain environment
    block_number: u64,
    timestamp: Timestamp,
    chain_id: String,
    canonical_address_length: usize,
    bech32_prefix: String,

    // for RpcContractInstance
}

const BLOCK_EPOCH: u64 = 1_000_000_000;
const WASM_MAGIC: [u8;4] = [0, 97, 115, 109];
const GZIP_MAGIC: [u8;4] = [0, 0, 0, 0];
const BASE_EOA: &str = "wasm1zcnn5gh37jxg9c6dp4jcjc7995ae0s5f5hj0lj";

fn maybe_unzip(input: Vec<u8>) -> Result<Vec<u8>, Error> {
    let magic = &input[0..4];
    if magic == WASM_MAGIC {
        Ok(input)
    }
    else if magic == GZIP_MAGIC {
        unimplemented!();
    }
    else {
        eprintln!("unidentifiable magic: {:?}", magic);
        unimplemented!();
    }
}

impl Model {
    fn new(rpc_url: &str, block_number: Option<u64>) -> Result<Self, Error> {
        let client = CwRpcClient::new(rpc_url, block_number)?;
        let block_number = client.block_number();
        let timestamp = client.timestamp()?;
        let chain_id = client.chain_id()?;
        Ok(Model {
            instances: HashMap::new(),
            bank: Bank::new()?,
            client: client,
            eoa: BASE_EOA.to_string(),

            block_number: block_number,
            timestamp: timestamp,
            chain_id: chain_id,
            canonical_address_length: 32,
            bech32_prefix: "wasm1".to_string()
        })
    }

    fn create_instance(&mut self, address: &str) -> Result<(), Error> {
        let deps = self.new_mock(address)?;
        let options = InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: false,
        };
        let contract_info = self.client.query_wasm_contract_info(address)?;
        let wasm_code = maybe_unzip(self.client.query_wasm_contract_code(contract_info.code_id)?)?;
        let inst = match cosmwasm_vm::Instance::from_code(wasm_code.as_slice(), deps, options, None) {
            Err(e) => {
                return Err(Error::vm_error(e));
            }
            Ok(i) => i,
        };
        let instance = RpcContractInstance::make_instance(inst);
        self.instances.insert(address.to_string(), instance);
        Ok(())
    }

    fn instantiate(&self, code_id: u64) -> Result<(), Error> {
        unimplemented!()
    }

    fn execute(&mut self, address: &str, msg: &[u8], funds: &[Coin]) -> Result<(), Error> {
        let env = self.env(address)?;
        let instance = match self.instances.get_mut(address) {
            Some(i) => i,
            None => {
                self.create_instance(address)?;
                self.instances.get_mut(address).unwrap()
            }
        };
        let sender = Addr::unchecked(&self.eoa);
        let response = instance.execute(&env, msg, &sender, funds)?;
        for resp in response.messages {
            println!("{:?}", resp);
        }
        self.update_blockchain_context();
        Ok(())
    }

    /// emulate blockchain block creation
    /// increment block number by 1
    /// increment timestamp by a constant
    fn update_blockchain_context(&mut self) {
        self.block_number += 1;
        self.timestamp.plus_nanos(BLOCK_EPOCH);
    }

    pub fn new_mock(&self, contract_address: &str) -> Result<RpcBackend, Error> {
        Ok(Backend {
            storage: self.mock_storage(contract_address)?,
            // is this correct?
            api: RpcMockApi::new(self.canonical_address_length),
            querier: RpcMockQuerier::new(&self.client),
        })
    }

    pub fn env(&self, contract_address: &str) -> Result<Env, Error> {
        Ok(Env {
            block: cosmwasm_std::BlockInfo {
                height: self.block_number,
                time: self.timestamp.clone(),
                chain_id: self.chain_id.clone(),
            },
            // assumption: all blocks have only 1 transaction
            transaction: Some(cosmwasm_std::TransactionInfo { index: 0 }),
            // I don't really know what this is for, so for now, set it to the target contract address
            contract: ContractInfo {
                address: Addr::unchecked(contract_address),
            },
        })
    }

    pub fn mock_storage(&self, contract_address: &str) -> Result<RpcMockStorage, Error> {
        let mut storage = RpcMockStorage::new();
        let states = self.client.query_wasm_contract_all(contract_address)?;
        for (k, v) in states {
            storage
                .set(k.as_slice(), v.as_slice())
                .0
                .map_err(|x| Error::vm_error(x))?;
        }
        Ok(storage)
    }
}

#[cfg(test)]
mod test {

    use cosmwasm_std::{Addr, Uint128, Coin};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::contract_vm::rpc_mock::model::Model;

    const MALAGA_RPC_URL: &'static str = "https://rpc.malaga-420.cosmwasm.com:443";
    const MALAGA_BLOCK_NUMBER: u64 = 2246678;
    const PAIR_ADDRESS: &'static str =
        "wasm15le5evw4regnwf9lrjnpakr2075fcyp4n4yzpelvqcuevzkw2lss46hslz";
    const TOKEN_ADDRESS: &'static str =
        "wasm124v54ngky9wxhx87t252x4xfgujmdsu7uhjdugtkkqt39nld0e6st7e64h";

    #[test]
    fn test_swap() {
        use serde_json::Value::Null;
        let mut model = Model::new(MALAGA_RPC_URL, Some(MALAGA_BLOCK_NUMBER)).unwrap();
        let prev_block_num = model.block_number;
        let msg_json = json!({
            "swap": {
            "offer_asset": {
                "info": { "native_token": { "denom": "umlg" } },
                "amount": "10"
            },
            "belief_price": Null,
            "max_spread": Null,
            "to": Null
            }
        });
        let msg_bytes = serde_json::to_string(&msg_json).unwrap();
        let funds = vec![Coin {
            denom: "umlg".to_string(),
            amount: Uint128::new(10),
        }];
        let _ = model.execute(PAIR_ADDRESS, msg_bytes.as_bytes(), &funds).unwrap();
        assert_eq!(model.block_number, prev_block_num + 1);
    }
}