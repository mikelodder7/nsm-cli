# nsm-cli

[![Apache 2.0 Licensed][license-image]][license-link]

A command line tool to be used within an AWS Nitro Enclave.

This tool can be used to run individual functions or started as a long-running process
that listens on a Unix Socket.

# Why this tool?

The documents for using the Nitro Enclave functions is hard and left up to the 
caller to dig through rust code. In addition, there are languages that must change how they operate
when using C-callable libraries. 

This tool allows a user to test using the functions provided by the Nitro Enclave,
see the results, and code according to expected outputs.

# Usage

## Generating random bytes

To read bytes from the nitro enclave trusted random number generator

```bash
nsm-cli rand <number-of-bytes>
```

`number-of-bytes` must be &lt; 256.

## PCR related functions

### Getting the current PCR hash

To retrieve the current hash of a specific PCR

```bash
nsm-cli describe-pcr <pcr-index>
```

Valid values for `pcr-index` are 0, 1, 2, 3, 4, 8.

### Adding the PCR hash

To add a value to an unlocked PCR

```bash
nsm-cli extend-pcr --index=<pcr-index> <pcr-hash>
```

Valid values for `pcr-index` are 3, 4. \[0, 1, 2, 8\] are locked when the enclave image is created.

PCR3 can be computed by running the following:

```bash
ROLEARN="arn:aws:iam::123456789012:role/Webserver"; \
python -c"import hashlib, sys; \
h=hashlib.sha384(); h.update(b'\0'*48); \
h.update(\"$ROLEARN\".encode('utf-8')); \
print(h.hexdigest())"
```

PCR4 can be computed by running the following:

```bash
INSTANCE_ID="i-1234567890abcdef0"; \
python -c"import hashlib, sys; \
h=hashlib.sha384(); h.update(b'\0'*48); \
h.update(\"$INSTANCE_ID\".encode('utf-8')); \
print(h.hexdigest())"
```
Nothing stops you from putting whatever you want for these inputs. All that matters is that
the `pcr-hash` is a hex encoded digest.

### Locking PCR hashes

Locking a PCR hash means it can no longer be changed. Once it is locked it cannot be unlocked.

**Locking a single PCR**

```bash
nsm-cli lock-pcr <pcr-index>
```

Valid values for `pcr-index` are 3, 4. \[0, 1, 2, 8\] are locked when the enclave image is created.

**Locking multiple PCRs**

```bash
nsm-cli lock-pcrs <pcr-range>
```

Valid values for `pcr-range` are 4, 5. \[0, `pcr-range`)

## Nitro Enclave Description

To get information about the Nitro Enclave

```bash
nsm-cli describe-nsm
```

## Attestations

Attestations include all locked PCRs and additional data supplied as input.

The total input to an attestation cannot exceed 4KB.

```bash
nsm-cli attestation --nonce=<nonce-baseurl> --public-key=<optional-base64url-asn.1-der> <optional-user-data-base64url>
```

The spec of the [attestation document](https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html)

## Server mode

The command line method is useful for debugging and testing, but many applications
would not want to wrap `nsm-cli` in a subprocess which interacts with a command line tool.
Instead, it's faster to start the process once and communicated with it. This can be done
by using running `nsm-cli` in server mode. Server mode listens on a unix socket
and listens for and responds with JSON messages.

```bash
nsm-cli server <unix-socket-path>
```

`unix-socket-path` can be any valid linux path.

Request messages are similar to the command line.

### Generating random bytes

Request
```json
{"number": <number-of-bytes>}
```

Response

Success Example

```json
{
  "ok": true,
  "value": "6bTyc59YDjCzgHUOdU_k_Bs2RiwroElZ6Gnxb68S7Y4"
}
```

Fail Example
```json
{
  "ok": false,
  "error":  {
    "msg": "Unable to initialize the file descriptor",
    "code": 8
  }
}
```

# Adding to docker image 
Build a docker image that adds nsm-cli to the docker container before converting to an
enclave image file

```dockerfile
FROM alpine:latest

COPY nsm-cli /bin/nsm-cli

CMD nsm-cli describe-nsm
```

# Installation
Download the binary and copy into your Docker image

```bash
curl -sSLO https://github.com/mikelodder7/nsm-cli/releases/download/v0.1.0/nsm-cli_x86_64-unknown-linux-musl
mv nsm-cli_x86_64-unknown-linux-musl nsm-cli
```

# Building from source

## Clone the project
1. git clone https://github.com/mikelodder7/nsm-cli
1. cd nsm-cli
1. git submodule init   
1. git submodule update --init
   
## Install Rust

1. curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -- -y
1. rustup target add x86_64-unknown-linux-musl

## Compiling the code

1. cargo build --target=x86_64-unknown-linux-musl --release
1. The binary is located in the target/x86_64-unknown-linux-musl/release folder

# Status

This project is **experimental** and may have bugs/memory safety issues.
*USE AT YOUR OWN RISK*

# Author

Michael Lodder

# License

Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

# References

- [Nitro SDK C Code](https://github.com/aws/aws-nitro-enclaves-sdk-c)
- [Nitro Rust API](https://github.com/aws/aws-nitro-enclaves-nsm-api)  
- [Nitro Enclave API](https://docs.aws.amazon.com/enclaves/latest/user/enclaves-user.pdf)
- [Nitro Enclave Docs](https://docs.aws.amazon.com/enclaves/latest/user/nitro-enclave.html)

[//]: # (badges)

[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-green.svg
[license-link]: https://github.com/mikelodder7/nsm-cli/blob/master/LICENSE