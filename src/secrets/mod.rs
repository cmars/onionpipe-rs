use keyring::Entry;
use thiserror::Error;
use torut::onion::TorSecretKeyV3;

#[derive(Error, Debug)]
pub enum SecretsError {
    #[error("secret storage error: {0}")]
    Keyring(#[from] keyring::error::Error),
    #[error("decode error: {0}")]
    Decode(#[from] base64::DecodeError),
}

pub type Result<T> = std::result::Result<T, SecretsError>;

pub trait SecretKeyStore {
    fn ensure_key(&self, petname: &str) -> Result<TorSecretKeyV3>;
    fn destroy_key(&self, petname: &str) -> Result<()>;
}

pub struct SecretKeyStorage {
    store: Box<dyn SecretKeyStore>,
}

impl SecretKeyStorage {
    fn new<T: SecretKeyStore + 'static>(store: T) -> Self {
        SecretKeyStorage {
            store: Box::new(store),
        }
    }

    pub fn ensure_key(&self, petname: &str) -> Result<TorSecretKeyV3> {
        self.store.ensure_key(petname)
    }

    pub fn destroy_key(&self, petname: &str) -> Result<()> {
        self.store.destroy_key(petname)
    }
}

impl std::default::Default for SecretKeyStorage {
    fn default() -> Self {
        SecretKeyStorage::new(KeyringStore {})
    }
}

struct KeyringStore {}

impl SecretKeyStore for KeyringStore {
    fn ensure_key(&self, petname: &str) -> Result<TorSecretKeyV3> {
        let entry = Entry::new_with_target(petname, "onionpipe", "")?;
        Ok(match entry.get_password() {
            Ok(ref key_str) => {
                let key: [u8; 64] = base64::decode(key_str)?.try_into().unwrap();
                TorSecretKeyV3::from(key)
            }
            Err(keyring::error::Error::NoEntry) => {
                let new_key = TorSecretKeyV3::generate();
                entry.set_password(&base64::encode(new_key.as_bytes()))?;
                TorSecretKeyV3::from(new_key)
            }
            Err(e) => return Err(SecretsError::Keyring(e)),
        })
    }

    fn destroy_key(&self, petname: &str) -> Result<()> {
        let entry = Entry::new_with_target(petname, "onionpipe", "")?;
        let result = entry.delete_password()?;
        Ok(result)
    }
}
