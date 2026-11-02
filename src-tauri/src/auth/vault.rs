use super::{AuthCode, AuthError, AuthResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Opaque protected bytes. Implementations must never fall back to plaintext.
pub trait CredentialVault: Send + Sync {
    fn read(&self) -> AuthResult<Option<Zeroizing<Vec<u8>>>>;
    fn write(&self, bytes: &[u8]) -> AuthResult<()>;
    fn clear(&self) -> AuthResult<()>;
}
pub struct WindowsVault {
    namespace: String,
}
#[derive(Serialize, Deserialize)]
struct Manifest {
    revision: String,
    parts: usize,
    sha256: String,
}
impl WindowsVault {
    pub fn production() -> Self {
        Self {
            namespace: "SheetsMonitor.OAuth.v2".into(),
        }
    }
    #[cfg(test)]
    fn isolated() -> Self {
        Self {
            namespace: format!("SheetsMonitor.Test.{}", uuid::Uuid::new_v4()),
        }
    }
    fn entry(&self, key: &str) -> AuthResult<keyring::Entry> {
        #[cfg(not(windows))]
        {
            let _ = key;
            return Err(AuthError::new(AuthCode::VaultUnavailable));
        }
        #[cfg(windows)]
        keyring::Entry::new(&self.namespace, key)
            .map_err(|_| AuthError::new(AuthCode::VaultUnavailable))
    }
    fn manifest(&self) -> AuthResult<Option<Manifest>> {
        match self.entry("current")?.get_secret() {
            Ok(bytes) => {
                let manifest: Manifest = serde_json::from_slice(&bytes)
                    .map_err(|_| AuthError::new(AuthCode::VaultCorrupt))?;
                if manifest.parts == 0
                    || manifest.parts > 128
                    || uuid::Uuid::parse_str(&manifest.revision).is_err()
                {
                    return Err(AuthError::new(AuthCode::VaultCorrupt));
                }
                Ok(Some(manifest))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AuthError::new(AuthCode::VaultUnavailable)),
        }
    }
    fn delete_parts(&self, manifest: &Manifest) -> AuthResult<()> {
        for index in 0..manifest.parts {
            match self
                .entry(&format!("{}:{index}", manifest.revision))?
                .delete_credential()
            {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(_) => return Err(AuthError::new(AuthCode::VaultUnavailable)),
            }
        }
        Ok(())
    }
}
impl CredentialVault for WindowsVault {
    fn read(&self) -> AuthResult<Option<Zeroizing<Vec<u8>>>> {
        let Some(manifest) = self.manifest()? else {
            return Ok(None);
        };
        let mut data = Zeroizing::new(Vec::new());
        for index in 0..manifest.parts {
            let bytes = Zeroizing::new(
                self.entry(&format!("{}:{index}", manifest.revision))?
                    .get_secret()
                    .map_err(|_| AuthError::new(AuthCode::VaultCorrupt))?,
            );
            if bytes.len() > 2048 {
                return Err(AuthError::new(AuthCode::VaultCorrupt));
            }
            data.extend_from_slice(&bytes);
        }
        if format!("{:x}", Sha256::digest(&data)) != manifest.sha256 {
            return Err(AuthError::new(AuthCode::VaultCorrupt));
        }
        Ok(Some(data))
    }
    fn write(&self, bytes: &[u8]) -> AuthResult<()> {
        if bytes.is_empty() || bytes.len() > 2048 * 128 {
            return Err(AuthError::new(AuthCode::VaultCorrupt));
        }
        let previous = self.manifest()?;
        let revision = uuid::Uuid::new_v4().to_string();
        let manifest = Manifest {
            revision,
            parts: bytes.len().div_ceil(2048),
            sha256: format!("{:x}", Sha256::digest(bytes)),
        };
        for (index, part) in bytes.chunks(2048).enumerate() {
            if self
                .entry(&format!("{}:{index}", manifest.revision))?
                .set_secret(part)
                .is_err()
            {
                let _ = self.delete_parts(&manifest);
                return Err(AuthError::new(AuthCode::VaultUnavailable));
            }
        }
        let encoded =
            serde_json::to_vec(&manifest).map_err(|_| AuthError::new(AuthCode::VaultCorrupt))?;
        if self.entry("current")?.set_secret(&encoded).is_err() {
            let _ = self.delete_parts(&manifest);
            return Err(AuthError::new(AuthCode::VaultUnavailable));
        }
        // Atomic current pointer publishes a complete new protected generation.
        // A cleanup failure leaves only encrypted, unreachable old chunks.
        if let Some(previous) = previous {
            let _ = self.delete_parts(&previous);
        }
        Ok(())
    }
    fn clear(&self) -> AuthResult<()> {
        let manifest = match self.manifest() {
            Ok(manifest) => manifest,
            Err(error) if error.code == AuthCode::VaultCorrupt => None,
            Err(error) => return Err(error),
        };
        // A corrupt pointer can be explicitly reset. Unknown old encrypted chunks
        // are never enumerated or mistaken for another application's credentials.
        match self.entry("current")?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err(AuthError::new(AuthCode::VaultUnavailable)),
        }
        if let Some(manifest) = manifest {
            self.delete_parts(&manifest)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "explicit Windows synthetic vault roundtrip; isolated UUID namespace, no Google"]
    fn windows_vault_roundtrip_chunks_and_deletes_only_its_test_namespace() {
        let vault = WindowsVault::isolated();
        let synthetic = Zeroizing::new(b"SYNTHETIC-NOT-A-GOOGLE-CREDENTIAL-".repeat(240));
        let result = (|| {
            assert!(vault.read()?.is_none());
            vault.write(&synthetic)?;
            assert_eq!(
                vault.read()?.as_deref().map(|v| v.as_slice()),
                Some(synthetic.as_slice())
            );
            vault.write(b"synthetic-replacement")?;
            assert_eq!(vault.read()?.unwrap().as_slice(), b"synthetic-replacement");
            Ok::<(), AuthError>(())
        })();
        let cleanup = vault.clear();
        result.unwrap();
        cleanup.unwrap();
        assert!(vault.read().unwrap().is_none());
    }
}
