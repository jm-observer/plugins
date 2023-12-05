use anyhow::{anyhow, bail};
use bytes::BytesMut;
use modbus_client::{Request, Response};
use models::{Dev, Single, Ty, Value, Var};
use serialport::{DataBits, Parity, StopBits};
use tokio_util::codec::Decoder;

wit_bindgen::generate!({
    // the name of the world in the `*.wit` input file
    world: "sensor",
    path: "../wits/sensor.wit",

    // For all exported worlds, interfaces, and resources, this specifies what
    // type they're corresponding to in this module. In this case the `MyHost`
    // struct defined below is going to define the exports of the `world`,
    // namely the `run` function.
    exports: {
        world: Sensor,
    },
});

struct Sensor;

impl Guest for Sensor {
    fn get_request_bytes(
        addr: u8,
        key: u8
    ) -> Result<
        wit_bindgen::rt::vec::Vec<u8>,
        wit_bindgen::rt::string::String
    > {
        get_request_bytes(addr, key).map_err(|x| x.to_string())
    }

    fn get_info() -> wit_bindgen::rt::vec::Vec<u8> {
        get_info()
    }

    fn get_rs(
        addr: u8,
        key: u8,
        data: wit_bindgen::rt::vec::Vec<u8>
    ) -> Result<
        Option<wit_bindgen::rt::vec::Vec<u8>>,
        wit_bindgen::rt::string::String
    > {
        get_rs(addr, key, data).map_err(|x| x.to_string())
    }
}

pub fn get_info() -> Vec<u8> {
    let var = Var {
        name:        "温度".to_string(),
        collect_key: 1,
        unit:        Some("摄氏度".to_string())
    };
    let dev = Dev {
        name: "20-485型大气压温湿度三合一变送器".to_string(),
        ty:   Ty::ModbusRtu(Single {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            parity:    Parity::None,
            stop_bits: StopBits::One,
            timeout:   200
        }),
        vars: vec![var]
    };
    dev.to_json().to_string().as_bytes().to_vec()
}

pub fn get_request_bytes(
    addr: u8,
    key: u8
) -> anyhow::Result<Vec<u8>> {
    let mut data = BytesMut::new();
    get_request(addr, key)?.to_bytes(&mut data);
    Ok(data.to_vec())
}

pub fn get_rs(
    addr: u8,
    key: u8,
    data: Vec<u8>
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut datas = BytesMut::from(data.as_slice());

    let mut res = get_request(addr, key)?;
    let Some(rs) = res.decode(&mut datas)? else {
        return Ok(None);
    };
    match key {
        1 => {
            let Response::ReadMultipleHoldingRegisters(_, _, rs) = rs
            else {
                bail!("not ReadMultipleHoldingRegisters");
            };
            let rs = rs.map_err(|x| anyhow!("{:?}", x))?;
            println!("{:02x?}", rs.get_values());
            let (Some(index), Some(index2)) =
                (rs.get_values().get(2), rs.get_values().get(3))
            else {
                bail!("get_values none");
            };
            Ok(Some(
                Value::F32(
                    u16::from_be_bytes([*index, *index2]) as f32
                        / 10.0
                )
                .to_json()?
                .to_string()
                .as_bytes()
                .to_vec()
            ))
        },
        _ => {
            bail!("key not 1");
        }
    }
}

fn get_request(addr: u8, key: u8) -> anyhow::Result<Request> {
    match key {
        1 => Ok(Request::read_multiple_holding_registers_request(
            addr, 0, 2
        )),
        _ => {
            bail!("unknow key {}", key);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::get_rs;

    #[test]
    fn test() {
        let data =
            [0x01u8, 0x03, 0x04, 0x03, 0x10, 0x00, 0xd7, 0xbb, 0xec]
                .to_vec();
        get_rs(1, 1, data).unwrap().unwrap();
    }
}
