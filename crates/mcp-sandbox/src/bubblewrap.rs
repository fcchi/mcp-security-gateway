use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, warn};
use crate::models::{NetworkAccess, SandboxConfig};

/// bubblewrapのラッパー
#[derive(Debug)]
pub struct BubblewrapWrapper {
    bwrap_path: PathBuf,
}

impl BubblewrapWrapper {
    /// 新しいBubblewrapWrapperを作成
    pub fn new() -> Option<Self> {
        // bubblewrapのパスをチェック
        let bwrap_path = which::which("bwrap").ok();
        
        if bwrap_path.is_none() {
            warn!("bubblewrap (bwrap) コマンドが見つかりませんでした。サンドボックスが無効になります。");
            return None;
        }
        
        debug!("bubblewrap found at: {:?}", bwrap_path);
        
        Some(Self {
            bwrap_path: bwrap_path.unwrap(),
        })
    }
    
    /// bubblewrapが使用可能かどうか
    pub fn is_available(&self) -> bool {
        true
    }
    
    /// bubblewrapコマンドを構築
    pub fn build_command(&self, config: &SandboxConfig, command: &str, args: &[String]) -> Command {
        let mut cmd = Command::new(&self.bwrap_path);
        
        // 基本的な分離設定
        cmd.arg("--unshare-all");
        cmd.arg("--die-with-parent");
        
        // ネットワーク設定
        match &config.network_access {
            NetworkAccess::None => {
                // ネットワークを完全に分離
                cmd.arg("--unshare-net");
            },
            NetworkAccess::Host => {
                // ネットワークを共有
                // デフォルトではunshare-allに含まれるので、何もしない
            },
            NetworkAccess::Restricted(hosts) => {
                // 制限付きネットワークは現在サポートしていないので、警告を出して無効化
                warn!("制限付きネットワークアクセスは現在サポートされていません: {:?}", hosts);
                cmd.arg("--unshare-net");
            }
        }
        
        // 読み書き可能なディレクトリをマウント
        for path in &config.rw_paths {
            cmd.arg("--bind");
            cmd.arg(path);
            cmd.arg(path);
        }
        
        // 読み取り専用ディレクトリをマウント
        for path in &config.ro_paths {
            cmd.arg("--ro-bind");
            cmd.arg(path);
            cmd.arg(path);
        }
        
        // 拒否するパスを空のディレクトリでマウント
        for path in &config.denied_paths {
            cmd.arg("--tmpfs");
            cmd.arg(path);
        }
        
        // seccompプロファイルの適用
        if let Some(seccomp_profile) = &config.seccomp_profile {
            cmd.arg("--seccomp");
            cmd.arg(seccomp_profile);
        }
        
        // 実行するコマンドとその引数を指定
        cmd.arg("--");
        cmd.arg(command);
        for arg in args {
            cmd.arg(arg);
        }
        
        // 標準入出力を設定
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        cmd
    }
} 