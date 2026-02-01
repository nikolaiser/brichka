use tokio::fs;
use std::env;

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub id: String,
    pub name: String
}

impl ClusterConfig {
    
    const CONFIG_FILE: &str = "brichka/cluster.json";

    fn local_path() -> String {
        let cwd = crate::CONTEXT.get().unwrap().cwd.to_owned();
        format!("{}/.{}", cwd, Self::CONFIG_FILE)
    }

    fn global_path() -> Result<String> {
        let home_dir = env::home_dir().context("Failed to locate the home directory")?;
        Ok(format!("{}/.config/{}", home_dir.to_string_lossy(), Self::CONFIG_FILE))
    }

    pub fn new(cluster: &crate::client::cluster::Cluster) -> ClusterConfig {
        ClusterConfig {
            id: cluster.id.to_owned(),
            name: cluster.name.to_owned()
        }
    }

    async fn read(path: String) -> Result<ClusterConfig> {
        let raw_json = fs::read_to_string(path).await?;
        serde_json::from_str(&raw_json).context("Failed to deserialize config")
    }

    pub async  fn read_local() -> Result<ClusterConfig> {
        Self::read(Self::local_path()).await
    }

    pub async  fn read_global() -> Result<ClusterConfig> {
        Self::read(Self::global_path()?).await
    }


    async fn write(&self, path: String) -> Result<()> {
        let raw_json = serde_json::to_string(self)?;
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, raw_json).await?;
        Ok(())
    }

    pub async fn write_local(&self) -> Result<()> {
        self.write(Self::local_path()).await
    }

    pub async fn write_global(&self) -> Result<()> {
        self.write(Self::global_path()?).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextConfig {
    pub id: String,
}

impl ContextConfig {
    
    const CONFIG_FILE: &str = "brichka/context.json";

    fn local_path() -> String {
        let cwd = crate::CONTEXT.get().unwrap().cwd.to_owned();
        format!("{}/.{}", cwd, Self::CONFIG_FILE)
    }

    pub fn new(id: String) -> ContextConfig {
        ContextConfig {
            id: id
        }
    }

    async fn read(path: String) -> Result<ContextConfig> {
        
        let raw_json = fs::read_to_string(path).await?;
        serde_json::from_str(&raw_json).context("Failed to deserialize config")
    }

    pub async fn read_local() -> Result<ContextConfig> {
        Self::read(Self::local_path()).await
    }


    async fn write(&self, path: String) -> Result<()> {
        let raw_json = serde_json::to_string(self)?;
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, raw_json).await?;
        Ok(())
    }

    pub async fn write_local(&self) -> Result<()> {
        self.write(Self::local_path()).await
    }

}

#[derive(Serialize, Deserialize)]
pub enum AuthConfig {
    DatabricksCli {
        path: String,    
        profile: String,
    },
    Token {
        value: String,
        host: String
    }
}


impl AuthConfig {
    
    const CONFIG_FILE: &str = "brichka/auth.json";

    fn global_path() -> Result<String> {
        let home_dir = env::home_dir().context("Failed to locate the home directory")?;
        Ok(format!("{}/.config/{}", home_dir.to_string_lossy(), Self::CONFIG_FILE))
    }


    async fn read(path: String) -> Result<AuthConfig> {
        let raw_json = fs::read_to_string(path).await?;
        serde_json::from_str(&raw_json).context("Failed to deserialize config")
    }

    pub async fn read_global() -> Result<AuthConfig> {
        Self::read(Self::global_path()?).await.or(Ok(AuthConfig::DatabricksCli { path: "databricks".to_string(), profile: "DEFAULT".to_string() }))
    }


    async fn write(&self, path: String) -> Result<()> {
        let raw_json = serde_json::to_string(self)?;
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, raw_json).await?;
        Ok(())
    }

    pub async fn write_global(&self) -> Result<()> {
        self.write(Self::global_path()?).await
    }
}
