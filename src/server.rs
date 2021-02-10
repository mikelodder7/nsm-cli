use crate::{args::Args, cmd::*, Error, ErrorCode, NsmResponse};
use std::os::unix::net::{UnixListener, UnixStream};

pub fn run(fd: i32, socket: String) -> Result<(), String> {
    let listener = UnixListener::bind(socket).map_err(|e| e.to_string())?;

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let response = match serde_json::from_reader::<&UnixStream, Args>(&stream) {
                Err(e) => NsmResponse {
                    ok: false,
                    error: Some(Error {
                        msg: e.to_string(),
                        code: ErrorCode::InvalidArgument,
                    }),
                    value: None,
                },
                Ok(req) => {
                    let response = match req {
                        Args::Rand { number } => nsm_get_random(fd, number),
                        Args::DescribeNSM => nsm_describe(fd),
                        Args::DescribePCR { index } => nsm_describe_pcr(fd, index),
                        Args::LockPCR { index } => nsm_lock_pcr(fd, index),
                        Args::LockPCRS { range } => nsm_lock_pcrs(fd, range),
                        Args::ExtendPCR { index, data } => nsm_extend_pcr(fd, index, data),
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

            // Ignore the result for now
            let _ = serde_json::to_writer(stream, &response);
        }
    }

    Ok(())
}
