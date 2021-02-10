#[macro_use]
extern crate serde_big_array;

big_array! { BigArray; }

mod args;
mod cmd;
mod server;

use args::Args;
use cmd::*;
use nsm_driver::{nsm_exit, nsm_init};
use nsm_io::{Digest, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use structopt::StructOpt;

#[derive(Serialize)]
pub struct Error {
    msg: String,
    #[serde(serialize_with = "write_error_code")]
    code: ErrorCode,
}

fn write_error_code<S>(v: &ErrorCode, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_u8(match *v {
        ErrorCode::Success => 0,
        ErrorCode::InvalidArgument => 1,
        ErrorCode::InvalidIndex => 2,
        ErrorCode::InvalidResponse => 3,
        ErrorCode::ReadOnlyIndex => 4,
        ErrorCode::InvalidOperation => 5,
        ErrorCode::BufferTooSmall => 6,
        ErrorCode::InputTooLarge => 7,
        ErrorCode::InternalError => 8,
    })
}

#[derive(Serialize)]
pub struct NsmResponse {
    pub(crate) ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<Error>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
}

impl From<Error> for NsmResponse {
    fn from(e: Error) -> Self {
        Self {
            ok: false,
            error: Some(e),
            value: None,
        }
    }
}

fn ok(value: String) -> NsmResponse {
    NsmResponse {
        ok: true,
        error: None,
        value: Some(value),
    }
}

type NsmResult<T> = Result<T, Error>;

#[derive(Deserialize, Serialize)]
pub struct NsmDescription {
    version_major: u16,
    version_minor: u16,
    version_patch: u16,
    #[serde(with = "BigArray")]
    module_id: [u8; 100],
    module_id_len: u32,
    max_pcrs: u16,
    #[serde(with = "BigArray")]
    locked_pcrs: [u16; 64],
    locked_pcrs_len: u32,
    digest: Digest,
}

impl Default for NsmDescription {
    fn default() -> NsmDescription {
        NsmDescription {
            version_major: 0,
            version_minor: 0,
            version_patch: 0,
            module_id: [0u8; 100],
            module_id_len: 0,
            max_pcrs: 0,
            locked_pcrs: [0u16; 64],
            locked_pcrs_len: 0,
            digest: Digest::SHA256,
        }
    }
}

fn main() {
    let args = Args::from_args();

    match get_nsm_fd() {
        Err(e) => {
            eprintln!("{}", e.msg);
            quit(0, e.into());
        }
        Ok(fd) => {
            let res = match args {
                Args::Attestation {
                    nonce,
                    public_key,
                    user_data,
                } => nsm_get_attestation_doc(fd, nonce, public_key, user_data),
                Args::ExtendPCR { index, data } => nsm_extend_pcr(fd, index, data),
                Args::LockPCR { index } => nsm_lock_pcr(fd, index),
                Args::LockPCRS { range } => nsm_lock_pcrs(fd, range),
                Args::DescribePCR { index } => nsm_describe_pcr(fd, index),
                Args::DescribeNSM => nsm_describe(fd),
                Args::Rand { number } => nsm_get_random(fd, number),
                Args::Server { socket } => match server::run(fd, socket) {
                    Ok(()) => Ok(ok("Server closed".to_string())),
                    Err(e) => Err(Error {
                        msg: e,
                        code: ErrorCode::InternalError,
                    }),
                },
            };
            match res {
                Ok(res) => quit(fd, res),
                Err(e) => quit(fd, e.into()),
            }
        }
    }
}

fn check_b64_arg(v: &mut Option<ByteBuf>, a: Option<String>, name: &str) -> NsmResult<()> {
    match a {
        None => Ok(()),
        Some(s) => match base64_url::decode(&s) {
            Ok(buf) => {
                if !buf.is_empty() {
                    *v = Some(ByteBuf::from(buf));
                }
                Ok(())
            }
            Err(e) => Err(Error {
                msg: format!("Invalid {} string: {:?}", name, e),
                code: ErrorCode::InvalidArgument,
            }),
        }
    }
}

fn get_nsm_fd() -> NsmResult<i32> {
    let fd = nsm_init();
    if fd < 0 {
        return Err(Error {
            msg: "Unable to initialize the file descriptor".to_string(),
            code: ErrorCode::InternalError,
        });
    }
    Ok(fd)
}

fn quit(fd: i32, res: NsmResponse) {
    if fd > 0 {
        nsm_exit(fd);
    }
    println!("{}", serde_json::to_string(&res).unwrap());
    if res.ok {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
