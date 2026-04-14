use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub qr_code_base64: String,
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct CredentialStatusResponse {
    pub has_password: bool,
    pub has_totp: bool,
    pub passkey_count: usize,
}
