use crate::error::ErrorHandler;
use crate::metrics;
use crate::proto::{
    self, CommandRequest, DeleteFileRequest, DeleteFileResponse, HealthRequest, HealthResponse,
    McpService, ReadFileRequest, ReadFileResponse, TaskCreatedResponse, TaskOutputChunk,
    TaskStatusRequest, TaskStatusResponse, WriteFileRequest, WriteFileResponse,
};
use dashmap::DashMap;
use mcp_common::error::{McpError, McpResult};
use mcp_policy::models::{CommandInfo, PolicyEngine, PolicyInput, UserInfo};
use mcp_sandbox::executor::CommandExecutor;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, info};
use uuid::Uuid;

/// MCPサービスの実装
#[derive(Debug)]
pub struct McpServiceImpl {
    policy_engine: PolicyEngine,
    command_executor: CommandExecutor,
    start_time: SystemTime,
    // タスク状態格納用（本実装ではRedis/PostgreSQLなどに置き換える）
    tasks: Arc<DashMap<String, proto::TaskInfo>>,
    results: Arc<DashMap<String, proto::TaskResult>>,
}

impl McpServiceImpl {
    /// 新しいサービスインスタンスを作成
    pub fn new(
        policy_engine: PolicyEngine,
        command_executor: CommandExecutor,
        start_time: SystemTime,
    ) -> Self {
        Self {
            policy_engine,
            command_executor,
            start_time,
            tasks: Arc::new(DashMap::new()),
            results: Arc::new(DashMap::new()),
        }
    }

    /// タスクIDを生成
    fn generate_task_id(&self) -> String {
        format!("task-{}", Uuid::new_v4().simple())
    }

    /// 現在のUNIXタイムスタンプを秒単位で取得
    #[allow(dead_code)]
    fn current_timestamp_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    /// ISO 8601形式の現在時刻文字列を取得
    fn current_iso8601(&self) -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

#[tonic::async_trait]
impl McpService for McpServiceImpl {
    /// ヘルスチェック
    async fn health(
        &self,
        request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        debug!("ヘルスチェックリクエスト: {:?}", request);

        // API呼び出しをメトリクスに記録
        metrics::increment_api_requests("GET", "/health", "200");

        // ErrorHandlerを使用して処理
        let result: McpResult<HealthResponse> = Ok({
            // 起動からの経過時間を計算
            let uptime = SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default()
                .as_secs();

            // エラー統計情報を取得
            let error_stats = ErrorHandler::get_error_stats();
            let total_errors: u64 = error_stats.values().sum();

            // 応答を作成
            let mut response = HealthResponse {
                status: "ok".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: uptime,
            };

            // 追加情報をメタデータに含める
            let metadata = request.metadata();
            if metadata.get("include-stats").is_some() {
                response.status = format!("ok [uptime={}s, errors={}]", uptime, total_errors);
            }

            response
        });

        ErrorHandler::handle(result)
    }

    /// コマンド実行
    async fn execute_command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<TaskCreatedResponse>, Status> {
        let req = request.into_inner();
        info!("コマンド実行リクエスト: command={}", req.command);

        // タスク実行時間の計測開始
        let timer = metrics::start_task_timer();

        // API呼び出しをメトリクスに記録
        metrics::increment_api_requests("POST", "/execute_command", "200");

        // ErrorHandlerを使用して実装全体を包む
        let result: McpResult<TaskCreatedResponse> = (|| {
            // ポリシーチェック
            let policy_timer = metrics::start_task_timer();
            let policy_input = PolicyInput {
                user: UserInfo {
                    id: "user1".to_string(), // TODO: 認証から取得
                    tenant_id: "tenant1".to_string(),
                    roles: vec!["user".to_string()],
                    attributes: HashMap::new(),
                },
                command: CommandInfo {
                    name: req.command.clone(),
                    args: req.args.clone(),
                    cwd: req.cwd.clone().unwrap_or_default(),
                    env: req.env.clone(),
                },
                file: None,
                network: None,
                resources: Default::default(),
                context: HashMap::new(),
            };

            // ポリシー評価
            let policy_result = self.policy_engine.check_command_execution(&policy_input);

            // ポリシー評価メトリクスを記録
            let policy_result_str = match &policy_result {
                Ok(_) => "allowed",
                Err(_) => "denied",
            };
            metrics::increment_policy_evaluations("command_execution", policy_result_str);
            metrics::observe_task_execution_time(
                policy_timer,
                "policy_evaluation",
                policy_result_str,
            );

            // エラーがあれば伝搬
            policy_result?;

            // タスクIDを生成
            let task_id = self.generate_task_id();
            let creation_time = self.current_iso8601();

            // タスク情報を保存
            let task_info = proto::TaskInfo {
                task_id: task_id.clone(),
                task_type: proto::TaskType::TaskCommand as i32,
                status: proto::TaskStatus::TaskCreated as i32,
                created_at: creation_time.clone(), // クローン
                started_at: None,
                completed_at: None,
                metadata: req.metadata.clone(),
            };

            self.tasks.insert(task_id.clone(), task_info.clone());

            // アクティブタスクをカウント
            metrics::increment_active_tasks();

            // 非同期でタスクを実行
            let executor = self.command_executor.clone();
            let tasks = self.tasks.clone();
            let results = self.results.clone();
            let cmd = req.command.clone();
            let args = req.args.clone();
            let env = req.env.clone();
            let cwd = req.cwd.clone();
            let timeout = if req.timeout > 0 {
                Some(req.timeout)
            } else {
                None
            };
            let task_id_clone = task_id.clone();

            // 別スレッドで実行
            tokio::spawn(async move {
                // サンドボックス実行時間の計測開始
                let sandbox_timer = metrics::start_sandbox_timer();

                // タスクを実行中に更新
                if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                    task.status = proto::TaskStatus::TaskRunning as i32;
                    task.started_at = Some(chrono::Utc::now().to_rfc3339());
                }

                // コマンドを実行
                let result = executor.execute(&cmd, args, env, cwd, timeout).await;

                // サンドボックス実行時間を記録
                metrics::observe_sandbox_execution_time(sandbox_timer, &cmd);

                // 結果を処理
                if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                    task.completed_at = Some(chrono::Utc::now().to_rfc3339());

                    match result {
                        Ok(output) => {
                            // 成功した場合
                            task.status = proto::TaskStatus::TaskCompleted as i32;

                            // 結果を保存
                            let task_result = proto::TaskResult {
                                exit_code: output.exit_code.unwrap_or(-1),
                                stdout: output.stdout,
                                stderr: output.stderr,
                                resource_usage: Some(proto::ResourceUsage {
                                    cpu_time_ms: output.resource_usage.cpu_time_ms,
                                    max_memory_kb: output.resource_usage.max_memory_kb,
                                    io_read_bytes: output.resource_usage.io_read_bytes,
                                    io_write_bytes: output.resource_usage.io_write_bytes,
                                }),
                                execution_time_ms: output.execution_time_ms,
                            };

                            results.insert(task_id_clone.clone(), task_result);

                            // 成功メトリクスを記録
                            metrics::observe_task_execution_time(
                                Instant::now() - Duration::from_millis(output.execution_time_ms),
                                "command",
                                "completed",
                            );
                        }
                        Err(e) => {
                            // 失敗した場合
                            task.status = proto::TaskStatus::TaskFailed as i32;

                            // エラーを記録
                            let error_type = match &e {
                                McpError::Execution(_) => "command_failed",
                                McpError::Temporary(_) => "timeout",
                                McpError::Sandbox(_) => "sandbox_error",
                                _ => "other",
                            };
                            metrics::increment_error_counter(error_type, &e.to_string());

                            // 失敗メトリクスを記録
                            metrics::observe_task_execution_time(
                                sandbox_timer,
                                "command",
                                "failed",
                            );

                            // 結果を保存
                            let task_result = proto::TaskResult {
                                exit_code: -1,
                                stdout: String::new(),
                                stderr: format!("Error: {}", e),
                                resource_usage: None,
                                execution_time_ms: 0,
                            };

                            results.insert(task_id_clone, task_result);
                        }
                    }

                    // アクティブタスクカウントを減少
                    metrics::decrement_active_tasks();
                }
            });

            // タスク作成応答を返す
            Ok(TaskCreatedResponse {
                task_id,
                status: proto::TaskStatus::TaskCreated as i32,
                created_at: creation_time,
            })
        })();

        // タスク作成の全体時間を記録
        let status = match &result {
            Ok(_) => "success",
            Err(_) => "error",
        };
        metrics::observe_task_execution_time(timer, "task_creation", status);

        // エラーハンドリングと応答
        ErrorHandler::handle(result)
    }

    /// タスク状態取得
    async fn get_task_status(
        &self,
        request: Request<TaskStatusRequest>,
    ) -> Result<Response<TaskStatusResponse>, Status> {
        let req = request.into_inner();
        debug!("タスク状態取得リクエスト: task_id={}", req.task_id);

        let result: McpResult<TaskStatusResponse> = (|| {
            // タスク情報を取得
            let task_info = match self.tasks.get(&req.task_id) {
                Some(info) => info.clone(),
                None => {
                    return Err(McpError::NotFound(format!(
                        "タスクが見つかりません: {}",
                        req.task_id
                    )))
                }
            };

            // 結果を取得（存在する場合）
            let result = self.results.get(&req.task_id).map(|r| r.clone());

            Ok(TaskStatusResponse {
                task_info: Some(task_info),
                result,
            })
        })();

        ErrorHandler::handle(result)
    }

    /// タスク出力ストリーミング
    type StreamTaskOutputStream = ReceiverStream<Result<TaskOutputChunk, Status>>;

    async fn stream_task_output(
        &self,
        request: Request<TaskStatusRequest>,
    ) -> Result<Response<Self::StreamTaskOutputStream>, Status> {
        let req = request.into_inner();
        debug!(
            "タスク出力ストリーミングリクエスト: task_id={}",
            req.task_id
        );

        let result: McpResult<Self::StreamTaskOutputStream> = (|| {
            // タスク情報を確認
            if !self.tasks.contains_key(&req.task_id) {
                return Err(McpError::NotFound(format!(
                    "タスクが見つかりません: {}",
                    req.task_id
                )));
            }

            // ダミーデータのストリームを作成（実際の実装ではコマンド出力を監視する）
            let (tx, rx) = tokio::sync::mpsc::channel(128);

            // ダミーデータ送信用のタスク
            let task_id = req.task_id.clone();
            tokio::spawn(async move {
                // 実装されたら、実際のコマンド出力をstreaming
                // ここでは単にダミーデータを送信
                let _ = tx
                    .send(Ok(TaskOutputChunk {
                        task_id: task_id.clone(),
                        r#type: proto::OutputChunkType::ChunkStdout as i32,
                        data: "ストリーミングテスト出力\n".as_bytes().to_vec(),
                        timestamp_ms: 0,
                    }))
                    .await;
            });

            Ok(ReceiverStream::new(rx))
        })();

        ErrorHandler::handle(result)
    }

    /// タスクキャンセル
    async fn cancel_task(
        &self,
        request: Request<TaskStatusRequest>,
    ) -> Result<Response<TaskStatusResponse>, Status> {
        let req = request.into_inner();
        info!("タスクキャンセルリクエスト: task_id={}", req.task_id);

        let result: McpResult<TaskStatusResponse> = (|| {
            // タスク情報を取得
            let mut task_info = match self.tasks.get_mut(&req.task_id) {
                Some(info) => info,
                None => {
                    return Err(McpError::NotFound(format!(
                        "タスクが見つかりません: {}",
                        req.task_id
                    )))
                }
            };

            // タスクをキャンセル状態に更新
            task_info.status = proto::TaskStatus::TaskCancelled as i32;
            task_info.completed_at = Some(self.current_iso8601());

            let task_info_clone = task_info.clone();

            // TODO: 実際のタスクをキャンセルする処理を実装

            // レスポンスを返す
            Ok(TaskStatusResponse {
                task_info: Some(task_info_clone),
                result: None,
            })
        })();

        ErrorHandler::handle(result)
    }

    /// ファイル読み取り
    async fn read_file(
        &self,
        request: Request<ReadFileRequest>,
    ) -> Result<Response<ReadFileResponse>, Status> {
        let req = request.into_inner();
        debug!("ファイル読み取りリクエスト: path={}", req.path);

        let result: McpResult<ReadFileResponse> = {
            // TODO: ここでポリシーチェックを行う

            // TODO: 実際のファイル読み取り実装
            Err(McpError::Internal(
                "ファイル読み取り機能は未実装です".to_string(),
            ))
        };

        ErrorHandler::handle(result)
    }

    /// ファイル書き込み
    async fn write_file(
        &self,
        request: Request<WriteFileRequest>,
    ) -> Result<Response<WriteFileResponse>, Status> {
        let req = request.into_inner();
        debug!("ファイル書き込みリクエスト: path={}", req.path);

        let result: McpResult<WriteFileResponse> = {
            // TODO: ここでポリシーチェックを行う

            // TODO: 実際のファイル書き込み実装
            Err(McpError::Internal(
                "ファイル書き込み機能は未実装です".to_string(),
            ))
        };

        ErrorHandler::handle(result)
    }

    /// ファイル削除
    async fn delete_file(
        &self,
        request: Request<DeleteFileRequest>,
    ) -> Result<Response<DeleteFileResponse>, Status> {
        let req = request.into_inner();
        debug!("ファイル削除リクエスト: path={}", req.path);

        let result: McpResult<DeleteFileResponse> = {
            // TODO: ここでポリシーチェックを行う

            // TODO: 実際のファイル削除実装
            Err(McpError::Internal(
                "ファイル削除機能は未実装です".to_string(),
            ))
        };

        ErrorHandler::handle(result)
    }
}
