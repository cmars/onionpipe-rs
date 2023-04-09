use std;
use std::path;
use std::{fs, io, result};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretsError {
    #[error("i/o error: {0}")]
    IO(#[from] io::Error),
}

pub type Result<T> = result::Result<T, SecretsError>;

pub struct SecretStore {
    secrets_dir: String,
}

const SERVICES_DIR: &'static str = "services";
const CLIENTS_DIR: &'static str = "clients";

impl SecretStore {
    pub fn new(secrets_dir: &str) -> SecretStore {
        SecretStore {
            secrets_dir: secrets_dir.to_owned(),
        }
    }

    pub fn ensure_service(&mut self, name: &str) -> Result<[u8; 64]> {
        let service_dir = path::PathBuf::from(&self.secrets_dir).join(SERVICES_DIR);
        if !service_dir.exists() {
            fs::create_dir_all(&service_dir)?;
        }
        let service_file = service_dir.join(name);
        if !service_file.exists() {
            let key = torut::onion::TorSecretKeyV3::generate().as_bytes();
            fs::write(&service_file, key)?;
            Ok(key)
        } else {
            let contents = fs::read(&service_file)?;
            let mut key = [0; 64];
            key.copy_from_slice(&contents);
            Ok(key)
        }
    }

    pub fn delete_service(&mut self, name: &str) -> Result<Option<()>> {
        let service_file = path::PathBuf::from(&self.secrets_dir)
            .join(SERVICES_DIR)
            .join(name);
        if service_file.exists() {
            fs::remove_file(&service_file)?;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub fn list_services(&self) -> Result<Vec<String>> {
        let mut services: Vec<String> = vec![];
        let service_file = path::PathBuf::from(&self.secrets_dir).join(SERVICES_DIR);
        let dir = fs::read_dir(service_file)?;
        for entry in dir {
            let entry = entry?;
            if let Some(fname) = entry.file_name().to_str() {
                services.push(fname.to_owned());
            }
        }
        Ok(services)
    }

    pub fn ensure_client(&mut self, name: &str) -> Result<[u8; 32]> {
        let client_dir = path::PathBuf::from(&self.secrets_dir).join(CLIENTS_DIR);
        if !client_dir.exists() {
            fs::create_dir_all(&client_dir)?;
        }
        let client_file = client_dir.join(name);
        if !client_file.exists() {
            let key = crypto_box::SecretKey::generate(&mut crypto_box::aead::OsRng)
                .as_bytes()
                .clone();
            fs::write(&client_file, key)?;
            Ok(key)
        } else {
            let contents = fs::read(&client_file)?;
            let mut key = [0; 32];
            key.copy_from_slice(&contents);
            Ok(key)
        }
    }

    pub fn delete_client(&mut self, name: &str) -> Result<Option<()>> {
        let client_file = path::PathBuf::from(&self.secrets_dir)
            .join(CLIENTS_DIR)
            .join(name);
        if client_file.exists() {
            fs::remove_file(&client_file)?;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub fn list_clients(&self) -> Result<Vec<String>> {
        let mut services: Vec<String> = vec![];
        let service_file = path::PathBuf::from(&self.secrets_dir).join(CLIENTS_DIR);
        let dir = fs::read_dir(service_file)?;
        for entry in dir {
            let entry = entry?;
            if let Some(fname) = entry.file_name().to_str() {
                services.push(fname.to_owned());
            }
        }
        Ok(services)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_service() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = SecretStore::new(secrets_dir.to_str().unwrap());
        let key1 = store.ensure_service("test").unwrap();
        assert!(secrets_dir.join(SERVICES_DIR).join("test").exists());
        let key2 = store.ensure_service("test").unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_delete_service() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = SecretStore::new(secrets_dir.to_str().unwrap());
        store.ensure_service("test").unwrap();
        assert!(secrets_dir.join("services").join("test").exists());
        let result = store.delete_service("test").unwrap();
        assert!(result.is_some());
        assert!(!secrets_dir.join("services").join("test").exists());
        let result = store.delete_service("test").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ensure_client() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = SecretStore::new(secrets_dir.to_str().unwrap());
        let key1 = store.ensure_client("test").unwrap();
        assert!(secrets_dir.join(CLIENTS_DIR).join("test").exists());
        let key2 = store.ensure_client("test").unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_delete_client() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = SecretStore::new(secrets_dir.to_str().unwrap());
        store.ensure_client("test").unwrap();
        assert!(secrets_dir.join("clients").join("test").exists());
        let result = store.delete_client("test").unwrap();
        assert!(result.is_some());
        assert!(!secrets_dir.join("clients").join("test").exists());
        let result = store.delete_client("test").unwrap();
        assert!(result.is_none());
    }
}
