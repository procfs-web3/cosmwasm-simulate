use cosmwasm_simulate::{Addr, Coin, DebugLog, Model, Timestamp, Uint128};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass]
struct ModelClass {
    inner: Model,
}

#[pyclass]
struct DebugLogClass {
    inner: DebugLog,
}

#[pymethods]
impl DebugLogClass {
    fn get_log(self_: PyRefMut<Self>) -> PyResult<Vec<String>> {
        let debug_log = &self_.inner;
        let mut out = Vec::new();
        for d in debug_log.logs.iter() {
            out.push(format!("{}", d));
        }
        Ok(out)
    }

    fn get_err_msg(self_: PyRefMut<Self>) -> PyResult<String> {
        let debug_log = &self_.inner;
        if let Some(err_msg) = &debug_log.err_msg {
            Ok(err_msg.to_string())
        }
        else {
            Ok("".to_string())
        }
    }

    fn get_stdout(self_: PyRefMut<Self>) -> PyResult<String> {
        let debug_log = &self_.inner;
        Ok(debug_log.get_stdout())
    }
}

#[pymethods]
impl ModelClass {
    #[new]
    fn new(
        rpc_url: String,
        block_number: Option<u64>,
        bech32_prefix: String,
    ) -> PyResult<ModelClass> {
        let model = Model::new(&rpc_url, block_number, &bech32_prefix)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(ModelClass { inner: model })
    }

    pub fn instantiate(
        mut self_: PyRefMut<Self>,
        code_id: u64,
        msg: &[u8],
        funds_: Vec<(String, u128)>,
    ) -> PyResult<DebugLogClass> {
        let model = &mut self_.inner;
        let funds: Vec<Coin> = funds_
            .iter()
            .map(|(d, a)| Coin {
                denom: d.to_string(),
                amount: Uint128::new(*a),
            })
            .collect();
        let debug_log = model
            .instantiate(code_id, msg, &funds)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(DebugLogClass { inner: debug_log })
    }

    pub fn execute(
        mut self_: PyRefMut<Self>,
        contract_addr_: &str,
        msg: &[u8],
        funds_: Vec<(String, u128)>,
    ) -> PyResult<DebugLogClass> {
        let model = &mut self_.inner;
        let funds: Vec<Coin> = funds_
            .iter()
            .map(|(d, a)| Coin {
                denom: d.to_string(),
                amount: Uint128::new(*a),
            })
            .collect();
        let contract_addr = Addr::unchecked(contract_addr_);
        let debug_log = model
            .execute(&contract_addr, msg, &funds)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(DebugLogClass { inner: debug_log })
    }

    pub fn cheat_block_number(mut self_: PyRefMut<Self>, block_number: u64) -> PyResult<()> {
        let model = &mut self_.inner;
        model
            .cheat_block_number(block_number)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    /// set latest block timestamp, units in nanoseconds
    pub fn cheat_block_timestamp(mut self_: PyRefMut<Self>, timestamp_: u64) -> PyResult<()> {
        let model = &mut self_.inner;
        let timestamp = Timestamp::from_nanos(timestamp_);
        model
            .cheat_block_timestamp(timestamp)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn cheat_code(mut self_: PyRefMut<Self>, contract_addr: &str, code: &[u8]) -> PyResult<()> {
        let model = &mut self_.inner;
        let contract_addr = Addr::unchecked(contract_addr);
        model
            .cheat_code(&contract_addr, code)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn cheat_message_sender(mut self_: PyRefMut<Self>, sender: &str) -> PyResult<()> {
        let model = &mut self_.inner;
        let sender_addr = Addr::unchecked(sender);
        model
            .cheat_message_sender(&sender_addr)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn cheat_storage(
        mut self_: PyRefMut<Self>,
        contract_addr: &str,
        key: &[u8],
        value: &[u8],
    ) -> PyResult<()> {
        let model = &mut self_.inner;
        let contract_addr = Addr::unchecked(contract_addr);
        model
            .cheat_storage(&contract_addr, key, value)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }
}

/// CosmWasm Simulator framework with Python bindings
#[pymodule]
fn cwsimpy(_py: Python, _m: &PyModule) -> PyResult<()> {
    Ok(())
}
