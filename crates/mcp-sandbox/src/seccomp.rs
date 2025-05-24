use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, error};

/// seccompプロファイルの種類
#[derive(Debug, Clone, Copy)]
pub enum SeccompProfileType {
    /// 基本的なプロファイル（ファイル操作と基本的なプロセス操作のみ許可）
    Basic,
    /// インターネットアクセスを許可するプロファイル
    Network,
}

/// seccompプロファイル管理
#[derive(Debug)]
pub struct SeccompProfileManager {
    profile_dir: PathBuf,
}

impl SeccompProfileManager {
    /// 新しいSeccompProfileManagerを作成
    pub fn new(profile_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&profile_dir).unwrap_or_else(|e| {
            error!("seccompプロファイルディレクトリの作成に失敗しました: {}", e);
        });
        
        Self { profile_dir }
    }
}

impl Default for SeccompProfileManager {
    fn default() -> Self {
        let tmp_dir = std::env::temp_dir().join("mcp-seccomp-profiles");
        Self::new(tmp_dir)
    }
}

impl SeccompProfileManager {
    /// seccompプロファイルのパスを取得
    pub fn get_profile_path(&self, profile_type: SeccompProfileType) -> McpResult<PathBuf> {
        let profile_name = match profile_type {
            SeccompProfileType::Basic => "basic.json",
            SeccompProfileType::Network => "network.json",
        };
        
        let profile_path = self.profile_dir.join(profile_name);
        
        if !profile_path.exists() {
            self.generate_profile(profile_type, &profile_path)?;
        }
        
        Ok(profile_path)
    }
    
    /// seccompプロファイルを生成
    fn generate_profile(&self, profile_type: SeccompProfileType, path: &PathBuf) -> McpResult<()> {
        let profile_content = match profile_type {
            SeccompProfileType::Basic => include_str!("../profiles/basic.json"),
            SeccompProfileType::Network => include_str!("../profiles/network.json"),
        };
        
        let mut file = File::create(path).map_err(|e| {
            error!("seccompプロファイルの作成に失敗しました: {}", e);
            McpError::Internal(format!("seccompプロファイルの作成に失敗しました: {}", e))
        })?;
        
        file.write_all(profile_content.as_bytes()).map_err(|e| {
            error!("seccompプロファイルの書き込みに失敗しました: {}", e);
            McpError::Internal(format!("seccompプロファイルの書き込みに失敗しました: {}", e))
        })?;
        
        debug!("seccompプロファイルを生成しました: {:?}", path);
        
        Ok(())
    }
}

use mcp_common::error::{McpError, McpResult}; 