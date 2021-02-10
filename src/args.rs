use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// The command line arguments
#[derive(Debug, Deserialize, Serialize, StructOpt)]
pub enum Args {
    /// Generate random bytes from /dev/nsm
    Rand {
        #[structopt(name = "number-of-bytes")]
        number: u8,
    },
    /// Sign an attestation document
    Attestation {
        /// Base64Url encoded
        #[structopt(long)]
        nonce: Option<String>,
        /// Base64Url encoded
        #[structopt(long)]
        public_key: Option<String>,
        /// Base64Url encoded
        #[structopt(long, name = "user-data")]
        user_data: Option<String>,
    },
    /// Run this process as long running process
    /// instead of run a single command and exit
    Server {
        #[structopt(long, name = "unix-socket-path")]
        /// The unix socket to listen on
        socket: String,
    },
    /// Read data from PlatformConfigurationRegister at index
    DescribePCR {
        #[structopt(long, name = "pcr-index")]
        /// index of the PCR to describe
        index: u16,
    },
    /// Extend PlatformConfigurationRegister at index with data
    ExtendPCR {
        #[structopt(short, long, name = "pcr-index")]
        /// Index the PCR to extend
        index: u16,
        #[structopt(long, name = "data")]
        /// Data to extend it with
        data: String,
    },
    /// Lock PlatformConfigurationRegister at index from further modifications
    LockPCR {
        #[structopt(long, name = "pcr-index")]
        /// Index to lock
        index: u16,
    },
    /// Lock PlatformConfigurationRegisters at indexes [0, range) from further modifications
    LockPCRS {
        #[structopt(long, name = "pcr-range")]
        /// Number of PCRs to lock, starting from index 0
        range: u16,
    },
    /// Return capabilities and version of the connected NitroSecureModule.
    /// Clients are recommended to decode major_version and minor_version first,
    /// and use an appropriate structure to hold this data,
    /// or fail if the version is not supported.
    DescribeNSM,
}
