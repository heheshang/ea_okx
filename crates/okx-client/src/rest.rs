//! REST API client implementation

pub struct OkxRestClient;

impl OkxRestClient {
    pub fn new(_credentials: crate::Credentials, _testnet: bool) -> crate::Result<Self> {
        Ok(Self)
    }
}
