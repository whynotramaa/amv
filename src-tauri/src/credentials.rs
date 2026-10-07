//! Windows-user-scoped DPAPI vault. Other platforms fail clearly by design.
#[cfg(windows)]
use anyhow::Context;
use anyhow::{bail, Result};
use std::path::PathBuf;
use zeroize::Zeroize;
use zeroize::Zeroizing;

#[derive(serde::Serialize, serde::Deserialize)]
struct ApiCredential {
    base_url: String,
    key: String,
}
impl Drop for ApiCredential {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

pub struct Credentials {
    #[cfg_attr(not(windows), allow(dead_code))]
    directory: PathBuf,
}

impl Credentials {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn set_api_key(&self, config: &crate::providers::ProviderConfig, key: &str) -> Result<()> {
        config.validate()?;
        let key = key.trim();
        if key.is_empty() || key.len() > 16 * 1024 || key.chars().any(char::is_control) {
            bail!("Enter a valid API key")
        }
        let value = ApiCredential {
            base_url: config.endpoint("")?.to_string(),
            key: key.into(),
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&value)?);
        self.set(config.provider.credential_target(), &bytes)
    }

    pub fn api_key(
        &self,
        config: &crate::providers::ProviderConfig,
    ) -> Result<Option<Zeroizing<String>>> {
        config.validate()?;
        let Some(bytes) = self.get(config.provider.credential_target())? else {
            return Ok(None);
        };
        if bytes.len() > 64 * 1024 {
            bail!("Credential record too large")
        }
        let mut value: ApiCredential = serde_json::from_slice(&bytes)?;
        if value.base_url != config.endpoint("")?.as_str() {
            return Ok(None);
        }
        if value.key.is_empty()
            || value.key.len() > 16 * 1024
            || value.key.chars().any(char::is_control)
        {
            bail!("Invalid stored API key")
        }
        Ok(Some(Zeroizing::new(std::mem::take(&mut value.key))))
    }

    /// A private, URL-bound accounting identity; the pepper remains in DPAPI.
    pub fn api_key_id(
        &self,
        config: &crate::providers::ProviderConfig,
        key: &str,
    ) -> Result<String> {
        config.validate()?;
        if key.is_empty() || key.len() > 16 * 1024 || key.chars().any(char::is_control) {
            bail!("Invalid API key");
        }
        static PEPPER_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = PEPPER_GATE
            .lock()
            .map_err(|_| anyhow::anyhow!("Credential identity lock unavailable"))?;
        let pepper = match self.get("api-usage-pepper")? {
            Some(value) => value,
            None => {
                let mut value = Zeroizing::new(vec![0u8; 32]);
                getrandom::fill(&mut value)
                    .map_err(|_| anyhow::anyhow!("Credential identity randomness unavailable"))?;
                self.set("api-usage-pepper", &value)?;
                value
            }
        };
        if pepper.len() != 32 {
            bail!("Invalid credential identity record");
        }
        Ok(derive_api_key_id(
            &pepper,
            config.provider.credential_target(),
            config.endpoint("")?.as_str(),
            key,
        ))
    }

    pub fn set(&self, slot: &str, secret: &[u8]) -> Result<()> {
        validate_slot(slot)?;
        if secret.is_empty() || secret.len() > 64 * 1024 {
            bail!("Invalid credential size")
        }
        #[cfg(windows)]
        {
            self.write_slot(slot, secret)
        }
        #[cfg(not(windows))]
        {
            bail!("Credential vault requires Windows")
        }
    }

    pub fn get(&self, slot: &str) -> Result<Option<Zeroizing<Vec<u8>>>> {
        validate_slot(slot)?;
        #[cfg(windows)]
        {
            self.read_slot(slot)
        }
        #[cfg(not(windows))]
        {
            bail!("Credential vault requires Windows")
        }
    }

    pub fn delete(&self, slot: &str) -> Result<()> {
        validate_slot(slot)?;
        #[cfg(windows)]
        {
            self.delete_slot(slot)
        }
        #[cfg(not(windows))]
        {
            bail!("Credential vault requires Windows")
        }
    }

    #[cfg(windows)]
    fn write_slot(&self, slot: &str, secret: &[u8]) -> Result<()> {
        std::fs::create_dir_all(&self.directory).context("Create credential vault directory")?;
        let encrypted = protect(slot, secret)?;
        let path = self.path(slot);
        let temp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
        let result = (|| {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?;
            file.write_all(&encrypted)?;
            file.sync_all()?;
            drop(file);
            atomic_replace(&temp, &path)
        })();
        let _ = std::fs::remove_file(&temp);
        result
    }

    #[cfg(windows)]
    fn read_slot(&self, slot: &str) -> Result<Option<Zeroizing<Vec<u8>>>> {
        let path = self.path(slot);
        if !path.exists() {
            return Ok(None);
        }
        if std::fs::metadata(&path)?.len() > 128 * 1024 {
            bail!("Credential record too large");
        }
        let value = std::fs::read(path).context("Read credential vault")?;
        if value.len() > 128 * 1024 {
            bail!("Credential record too large");
        }
        unprotect(slot, &value).map(Some)
    }

    #[cfg(windows)]
    fn delete_slot(&self, slot: &str) -> Result<()> {
        match std::fs::remove_file(self.path(slot)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).context("Delete credential vault slot"),
        }
    }

    #[cfg(windows)]
    fn path(&self, slot: &str) -> PathBuf {
        self.directory.join(format!("{slot}.dpapi"))
    }
}

fn derive_api_key_id(pepper: &[u8], provider: &str, url: &str, key: &str) -> String {
    use base64::Engine as _;
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"Harness-API-usage-v1");
    hash.update(pepper);
    for part in [provider, url, key] {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash.finalize())
}

fn validate_slot(slot: &str) -> Result<()> {
    if slot.is_empty()
        || slot.len() > 128
        || !slot
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        bail!("Invalid credential slot")
    }
    Ok(())
}

#[cfg(windows)]
fn protect(slot: &str, secret: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB {
        cbData: secret.len() as u32,
        pbData: secret.as_ptr() as *mut _,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let mut entropy_bytes = Zeroizing::new(format!("AMV-DPAPI-v1:{slot}").into_bytes());
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: entropy_bytes.len() as u32,
        pbData: entropy_bytes.as_mut_ptr(),
    };
    unsafe {
        CryptProtectData(
            &input,
            None,
            Some(&entropy),
            None,
            None,
            Default::default(),
            &mut output,
        )
    }
    .context("DPAPI encryption failed")?;
    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(
            output.pbData as *mut _,
        )));
    }
    Ok(result)
}

#[cfg(windows)]
fn unprotect(slot: &str, encrypted: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_ptr() as *mut _,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let mut entropy_bytes = Zeroizing::new(format!("AMV-DPAPI-v1:{slot}").into_bytes());
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: entropy_bytes.len() as u32,
        pbData: entropy_bytes.as_mut_ptr(),
    };
    unsafe {
        CryptUnprotectData(
            &input,
            None,
            Some(&entropy),
            None,
            None,
            Default::default(),
            &mut output,
        )
    }
    .context("DPAPI decryption failed")?;
    let plaintext =
        unsafe { std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize) };
    let result = Zeroizing::new(plaintext.to_vec());
    plaintext.zeroize();
    unsafe {
        windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(
            output.pbData as *mut _,
        )));
    }
    Ok(result)
}

#[cfg(windows)]
fn atomic_replace(source: &std::path::Path, target: &std::path::Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            windows::core::PCWSTR(source.as_ptr()),
            windows::core::PCWSTR(target.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .context("Atomically replace credential vault")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slots_cannot_escape_vault() {
        for slot in ["", "../token", "has space", "a/b", "é"] {
            assert!(validate_slot(slot).is_err());
        }
        assert!(validate_slot("chatgpt-account-1").is_ok());
        assert!(validate_slot(&"a".repeat(129)).is_err());
    }
    #[test]
    fn non_windows_vault_is_explicitly_unsupported() {
        #[cfg(not(windows))]
        assert!(Credentials::new(PathBuf::from("unused"))
            .get("token")
            .is_err());
    }

    #[test]
    fn accounting_identity_is_stable_and_bound() {
        let id = derive_api_key_id(&[1; 32], "gemini", "https://example.com/v1/", "key");
        assert_eq!(id.len(), 43);
        assert_eq!(
            id,
            derive_api_key_id(&[1; 32], "gemini", "https://example.com/v1/", "key")
        );
        for other in [
            derive_api_key_id(&[2; 32], "gemini", "https://example.com/v1/", "key"),
            derive_api_key_id(&[1; 32], "deepseek", "https://example.com/v1/", "key"),
            derive_api_key_id(&[1; 32], "gemini", "https://other.com/v1/", "key"),
            derive_api_key_id(&[1; 32], "gemini", "https://example.com/v1/", "other"),
        ] {
            assert_ne!(id, other);
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_vault_roundtrip_replaces_binds_and_deletes() {
        struct TempDir(PathBuf);
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        let directory =
            TempDir(std::env::temp_dir().join(format!("amv-credentials-{}", uuid::Uuid::new_v4())));
        let credentials = Credentials::new(directory.0.clone());
        let config = crate::providers::defaults().remove(0);
        let slot = config.provider.credential_target();
        let path = directory.0.join(format!("{slot}.dpapi"));

        credentials.set_api_key(&config, "test-api-key").unwrap();
        assert_eq!(
            credentials
                .api_key(&config)
                .unwrap()
                .as_deref()
                .map(String::as_str),
            Some("test-api-key")
        );
        let encrypted = std::fs::read(&path).unwrap();
        assert!(!encrypted
            .windows("test-api-key".len())
            .any(|bytes| bytes == b"test-api-key"));

        credentials.set_api_key(&config, "replacement-key").unwrap();
        assert_eq!(
            credentials
                .api_key(&config)
                .unwrap()
                .as_deref()
                .map(String::as_str),
            Some("replacement-key")
        );
        assert_eq!(
            std::fs::read_dir(&directory.0)
                .unwrap()
                .filter_map(Result::ok)
                .count(),
            1
        );

        let mut changed = config.clone();
        changed.base_url.push_str("changed/");
        assert!(credentials.api_key(&changed).unwrap().is_none());

        let identity = credentials.api_key_id(&config, "replacement-key").unwrap();
        assert_ne!(
            identity,
            credentials.api_key_id(&changed, "replacement-key").unwrap()
        );
        credentials.delete(slot).unwrap();
        assert!(credentials.api_key(&config).unwrap().is_none());
        credentials.delete(slot).unwrap();
        assert!(credentials.get(slot).unwrap().is_none());
        credentials.set_api_key(&config, "replacement-key").unwrap();
        assert_eq!(
            identity,
            credentials.api_key_id(&config, "replacement-key").unwrap()
        );
        credentials.delete(slot).unwrap();
        assert_eq!(
            credentials.get("api-usage-pepper").unwrap().unwrap().len(),
            32
        );
    }
}
