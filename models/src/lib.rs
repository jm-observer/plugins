use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number};
use serialport::{DataBits, Parity, StopBits};

#[derive(Deserialize, Debug)]
pub struct Dev {
    pub name: String,
    pub ty:   Ty,
    pub vars: Vec<Var>
}
impl Dev {
    pub fn to_json(self) -> serde_json::Value {
        let mut obj = Map::new();
        obj.insert("name".to_string(), self.name.into());
        obj.insert("ty".to_string(), self.ty.to_json());
        let vars: Vec<serde_json::Value> =
            self.vars.into_iter().map(|x| x.to_json()).collect();
        obj.insert("vars".to_string(), vars.into());
        obj.into()
    }
}
#[derive(Deserialize, Debug)]
pub struct Var {
    pub name:        String,
    pub collect_key: usize,
    pub unit:        Option<String>
}
impl Var {
    pub fn to_json(self) -> serde_json::Value {
        let mut obj = Map::new();
        obj.insert("name".to_string(), self.name.into());
        obj.insert(
            "collect_key".to_string(),
            self.collect_key.into()
        );
        match self.unit {
            None => {},
            Some(unit) => {
                obj.insert("unit".to_string(), unit.into());
            }
        }
        obj.into()
    }
}

pub enum Value {
    F32(f32)
}

impl Value {
    pub fn to_json(self) -> anyhow::Result<serde_json::Value> {
        match self {
            Value::F32(single) => {
                let mut obj = Map::new();
                let Some(num) = Number::from_f64(single as f64)
                else {
                    bail!("Number::from_f64 fail: {}", single);
                };
                obj.insert(
                    "F32".to_string(),
                    serde_json::Value::Number(num)
                );
                Ok(obj.into())
            }
        }
    }
}
#[derive(Deserialize, Debug)]

pub enum Ty {
    ModbusRtu(Single)
}

impl Ty {
    pub fn to_json(self) -> serde_json::Value {
        match self {
            Ty::ModbusRtu(single) => {
                let mut obj = Map::new();
                obj.insert("ModbusRtu".to_string(), single.to_json());
                obj.into()
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Single {
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub parity:    Parity,
    pub stop_bits: StopBits,
    pub timeout:   u64
}

impl Single {
    pub fn to_json(self) -> serde_json::Value {
        let mut obj = Map::new();
        obj.insert("baud_rate".to_string(), self.baud_rate.into());
        obj.insert(
            "data_bits".to_string(),
            self.data_bits.to_string().into()
        );
        obj.insert(
            "parity".to_string(),
            self.parity.to_string().into()
        );
        obj.insert(
            "stop_bits".to_string(),
            self.stop_bits.to_string().into()
        );
        obj.insert("timeout".to_string(), self.timeout.into());
        obj.into()
    }
}
