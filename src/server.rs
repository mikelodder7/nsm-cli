use crate::{args::Args, cmd::*, Error, ErrorCode, NsmResponse};
use serde::Deserialize;
use std::io::Write;
use std::os::unix::net::UnixListener;

pub fn run(fd: i32, socket: &str) -> Result<(), String> {
    println!("Binding on {}", socket);
    let listener = UnixListener::bind(socket).map_err(|e| e.to_string())?;

    println!("Listening on {}", socket);
    loop {
        let (stream, addr) = listener.accept().map_err(|e| e.to_string())?;
        println!("Client on {:?}", addr);
        let mut writer = stream.try_clone().unwrap();
        let mut de = serde_json::Deserializer::from_reader(stream);
        loop {
            let response = match Args::deserialize(&mut de) {
                Err(e) => match e.classify() {
                    serde_json::error::Category::Io => {
                        println!("Connection on {:?} closed", addr);
                        break;
                    }
                    _ => NsmResponse {
                        ok: false,
                        error: Some(Error {
                            msg: e.to_string(),
                            code: ErrorCode::InvalidArgument,
                        }),
                        value: None,
                    },
                },
                Ok(req) => {
                    println!("Received request: {:?}", req);
                    let response = match req {
                        Args::Rand { number } => nsm_get_random(fd, number),
                        Args::DescribeNsm => nsm_describe(fd),
                        Args::DescribePcr { index } => nsm_describe_pcr(fd, index),
                        Args::LockPcr { index } => nsm_lock_pcr(fd, index),
                        Args::LockPcrs { range } => nsm_lock_pcrs(fd, range),
                        Args::ExtendPcr { index, data } => nsm_extend_pcr(fd, index, data),
                        Args::Attestation {
                            nonce,
                            public_key,
                            user_data,
                        } => nsm_get_attestation_doc(fd, nonce, public_key, user_data),
                        _ => Err(Error {
                            msg: "Invalid request".to_string(),
                            code: ErrorCode::InvalidOperation,
                        }),
                    };
                    match response {
                        Ok(r) => r,
                        Err(e) => e.into(),
                    }
                }
            };

            println!("Writing response from nsm");
            match serde_json::to_vec(&response) {
                Err(e) => {
                    println!("{:?}", e);
                }
                Ok(s) => {
                    if let Err(e) = writer.write(s.as_slice()) {
                        println!("{:?}", e);
                    }
                }
            }
            if let Err(e) = writer.flush() {
                println!("{:?}", e);
            }
        }
    }
}
