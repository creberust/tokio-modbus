// SPDX-FileCopyrightText: Copyright (c) 2017-2022 slowtec GmbH <post@slowtec.de>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RTU server example with slave address filtering and optional response

use std::{thread, time::Duration};

use tokio_modbus::{prelude::*, server::rtu::Server};

struct Service {
    slave: Slave,
}

#[async_trait::async_trait]
impl tokio_modbus::server::Service for Service {
    type Request = SlaveRequest<'static>;

    async fn call(&self, req: Self::Request) -> Result<Response, Exception> {
        if req.slave != self.slave.into() {
            return Err(Exception::IllegalFunction);
        }
        match req.request {
            Request::ReadInputRegisters(_addr, cnt) => {
                let mut registers = vec![0; cnt.into()];
                registers[2] = 0x77;
                Ok(Response::ReadInputRegisters(registers))
            }
            _ => Err(Exception::IllegalFunction),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slave = Slave(12);
    let builder = tokio_serial::new("/dev/ttyS10", 19200);
    let server_serial = tokio_serial::SerialStream::open(&builder).unwrap();

    println!("Starting up server...");
    let _server = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = Server::new(server_serial);
        let service = Service { slave };
        rt.block_on(async {
            if let Err(err) = server.serve_forever(service).await {
                eprintln!("{err}");
            }
        });
    });

    // Give the server some time for stating up
    thread::sleep(Duration::from_secs(1));

    println!("CLIENT: Connecting client...");
    let client_serial = tokio_serial::SerialStream::open(&builder).unwrap();
    let mut ctx = rtu::attach_slave(client_serial, slave);
    println!("CLIENT: Reading input registers...");
    let rsp = ctx.read_input_registers(0x00, 7).await?;
    println!("CLIENT: The result is '{rsp:#x?}'");
    assert_eq!(rsp.unwrap(), vec![0x0, 0x0, 0x77, 0x0, 0x0, 0x0, 0x0]);

    println!("CLIENT: Reading with illegal function... (should return IllegalFunction)");
    let response = ctx.read_holding_registers(0x100, 1).await.unwrap();
    println!("CLIENT: The result is '{response:?}'");
    assert!(matches!(response, Err(Exception::IllegalFunction)));

    println!("CLIENT: Done.");

    Ok(())
}
