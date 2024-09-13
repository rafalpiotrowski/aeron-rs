pub trait CredentialSupplier {
    /// Provide a credential to be included in Session Connect message to a system.
    /// returns a credential in binary form to be included in the Session Connect message to system.
    fn encoded_credentials(&self) -> &[u8];

    /// Given some encoded challenge data, provide the credentials to be included in a Challenge Response as part of
    /// authentication with a system.
    ///
    /// endcoded_challenge from the cluster to use in providing a credential.
    ///
    /// returns encoded credentials in binary form to be included in the Challenge Response to the system.
    fn on_challenge(&self, endcoded_challenge: &[u8]) -> &[u8];
}

pub struct NoCredentialsSupplier;

impl CredentialSupplier for NoCredentialsSupplier {
    fn encoded_credentials(&self) -> &[u8] {
        &[]
    }

    fn on_challenge(&self, _endcoded_challenge: &[u8]) -> &[u8] {
        &[]
    }
}
