use crate::{check_b64_arg, ok, Error, NsmDescription, NsmResponse, NsmResult};
use nsm_driver::nsm_process_request;
use nsm_io::{Digest, ErrorCode, Request, Response};

pub fn nsm_get_attestation_doc(
    fd: i32,
    n: Option<String>,
    pk: Option<String>,
    ud: Option<String>,
) -> NsmResult<NsmResponse> {
    let mut nonce = None;
    check_b64_arg(&mut nonce, n, "nonce")?;
    let mut public_key = None;
    check_b64_arg(&mut public_key, pk, "public key")?;
    let mut user_data = None;
    check_b64_arg(&mut user_data, ud, "user data")?;

    let request = Request::Attestation {
        nonce,
        user_data,
        public_key,
    };
    match nsm_process_request(fd, request) {
        Response::Attestation { document } => Ok(ok(base64_url::encode(document.as_slice()))),
        Response::Error(err) => Err(Error {
            msg: "Unable to generate attestation document".to_string(),
            code: err,
        }),
        _ => Err(Error {
            msg: "Unable to generate attestation document".to_string(),
            code: ErrorCode::InvalidResponse,
        }),
    }
}

pub fn nsm_get_random(fd: i32, number: u8) -> NsmResult<NsmResponse> {
    match nsm_process_request(fd, Request::GetRandom) {
        Response::GetRandom { random } => Ok(ok(base64_url::encode(&random[..(number as usize)]))),
        Response::Error(err) => Err(Error {
            msg: "Unable to read random bytes".to_string(),
            code: err,
        }),
        e => Err(Error {
            msg: format!("{:?}", e),
            code: ErrorCode::InvalidResponse,
        }),
    }
}

pub fn nsm_extend_pcr(fd: i32, index: u16, d: String) -> NsmResult<NsmResponse> {
    let mut data = vec![];
    match base64_url::decode(&d) {
        Ok(v) => data.extend_from_slice(v.as_slice()),
        Err(_) => {
            return Err(Error {
                msg: "Invalid data supplied".to_string(),
                code: ErrorCode::InvalidArgument,
            })
        }
    };

    let request = Request::ExtendPCR { index, data };

    match nsm_process_request(fd, request) {
        Response::ExtendPCR { data: pcr } => Ok(ok(base64_url::encode(pcr.as_slice()))),
        Response::Error(e) => Err(Error {
            msg: "Unable to extend pcr".to_string(),
            code: e,
        }),
        _ => Err(Error {
            msg: "Unable to extend pcr".to_string(),
            code: ErrorCode::InvalidResponse,
        }),
    }
}

pub fn nsm_lock_pcr(fd: i32, index: u16) -> NsmResult<NsmResponse> {
    let request = Request::LockPCR { index };

    match nsm_process_request(fd, request) {
        Response::LockPCR => Ok(ok(String::new())),
        Response::Error(err) => Err(Error {
            code: err,
            msg: "Unable to lock pcr".to_string(),
        }),
        _ => Err(Error {
            code: ErrorCode::InvalidResponse,
            msg: "Unable to lock pcr".to_string(),
        }),
    }
}

pub fn nsm_lock_pcrs(fd: i32, range: u16) -> NsmResult<NsmResponse> {
    let request = Request::LockPCRs { range };

    match nsm_process_request(fd, request) {
        Response::LockPCRs => Ok(ok(String::new())),
        Response::Error(err) => Err(Error {
            code: err,
            msg: "Unable to lock pcrs".to_string(),
        }),
        _ => Err(Error {
            code: ErrorCode::InvalidResponse,
            msg: "Unable to lock pcrs".to_string(),
        }),
    }
}

pub fn nsm_describe_pcr(fd: i32, index: u16) -> NsmResult<NsmResponse> {
    let request = Request::DescribePCR { index };

    match nsm_process_request(fd, request) {
        Response::DescribePCR { lock, data } => {
            let msg = format!(
                r#"{{"lock":{},"data":"{}"}}"#,
                lock,
                base64_url::encode(data.as_slice())
            );
            Ok(ok(base64_url::encode(&msg.as_bytes())))
        }
        Response::Error(err) => Err(Error {
            code: err,
            msg: "Unable to describe pcr".to_string(),
        }),
        _ => Err(Error {
            code: ErrorCode::InvalidResponse,
            msg: "Unable to describe pcr".to_string(),
        }),
    }
}

pub fn nsm_describe(fd: i32) -> NsmResult<NsmResponse> {
    let request = Request::DescribeNSM;

    match nsm_process_request(fd, request) {
        Response::DescribeNSM {
            version_major,
            version_minor,
            version_patch,
            module_id,
            max_pcrs,
            locked_pcrs,
            digest,
        } => {
            let mut nsm_description = NsmDescription::default();
            nsm_description.version_major = version_major;
            nsm_description.version_minor = version_minor;
            nsm_description.version_patch = version_patch;
            nsm_description.max_pcrs = max_pcrs;

            match digest {
                Digest::SHA256 => {
                    nsm_description.digest = Digest::SHA256;
                }
                Digest::SHA384 => {
                    nsm_description.digest = Digest::SHA384;
                }
                Digest::SHA512 => {
                    nsm_description.digest = Digest::SHA512;
                }
            }
            nsm_description.locked_pcrs_len = locked_pcrs.len() as u32;

            for (i, val) in locked_pcrs.iter().enumerate() {
                nsm_description.locked_pcrs[i] = *val;
            }

            let module_id_len = std::cmp::min(nsm_description.module_id.len() - 1, module_id.len());
            nsm_description.module_id[0..module_id_len]
                .copy_from_slice(&module_id.as_bytes()[0..module_id_len]);
            nsm_description.module_id[module_id_len] = 0;
            nsm_description.module_id_len = module_id_len as u32;

            Ok(ok(serde_json::to_string(&nsm_description).unwrap()))
        }
        Response::Error(err) => Err(Error {
            code: err,
            msg: "Unable to describe nsm".to_string(),
        }),
        _ => Err(Error {
            code: ErrorCode::InvalidResponse,
            msg: "Unable to describe nsm".to_string(),
        }),
    }
}
