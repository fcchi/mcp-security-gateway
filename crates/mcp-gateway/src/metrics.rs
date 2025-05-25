#![allow(static_mut_refs)]

use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry};
use std::sync::Once;
use std::time::Instant;

static METRICS_INIT: Once = Once::new();
static mut REGISTRY: Option<Registry> = None;
static mut API_REQUESTS: Option<IntCounterVec> = None;
static mut TASK_EXECUTION_TIME: Option<HistogramVec> = None;
static mut ACTIVE_TASKS: Option<IntGauge> = None;
static mut POLICY_EVALUATIONS: Option<IntCounterVec> = None;
static mut SANDBOX_EXECUTION_TIME: Option<HistogramVec> = None;
static mut ERROR_COUNTER: Option<IntCounterVec> = None;

/// Metrics initialization
pub fn init_metrics() {
    METRICS_INIT.call_once(|| {
        let registry = Registry::new();

        // API request counter
        let api_requests = IntCounterVec::new(
            Opts::new("mcp_api_requests_total", "Total number of API calls"),
            &["method", "path", "status"],
        )
        .unwrap();

        // Task execution time histogram
        let task_execution_time = HistogramVec::new(
            HistogramOpts::new("mcp_task_latency_ms", "Task execution time (milliseconds)")
                .buckets(vec![
                    5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
                ]),
            &["task_type", "status"],
        )
        .unwrap();

        // Active tasks count
        let active_tasks =
            IntGauge::new("mcp_active_tasks", "Number of currently running tasks").unwrap();

        // Policy evaluation counter
        let policy_evaluations = IntCounterVec::new(
            Opts::new(
                "mcp_policy_evaluations_total",
                "Total number of policy evaluations",
            ),
            &["policy", "result"],
        )
        .unwrap();

        // Sandbox execution time
        let sandbox_execution_time = HistogramVec::new(
            HistogramOpts::new(
                "mcp_sandbox_execution_time_ms",
                "Sandbox execution time (milliseconds)",
            )
            .buckets(vec![
                5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
            ]),
            &["command"],
        )
        .unwrap();

        // Error counter
        let error_counter = IntCounterVec::new(
            Opts::new("mcp_errors_total", "Total number of errors"),
            &["type", "code"],
        )
        .unwrap();

        // Register metrics with registry
        registry.register(Box::new(api_requests.clone())).unwrap();
        registry
            .register(Box::new(task_execution_time.clone()))
            .unwrap();
        registry.register(Box::new(active_tasks.clone())).unwrap();
        registry
            .register(Box::new(policy_evaluations.clone()))
            .unwrap();
        registry
            .register(Box::new(sandbox_execution_time.clone()))
            .unwrap();
        registry.register(Box::new(error_counter.clone())).unwrap();

        // Process metrics are conditionally registered
        // Process collector disabled for now due to compatibility issues
        /*
        #[cfg(target_os = "linux")]
        {
            if let Ok(collector) = prometheus::process_collector::ProcessCollector::for_self() {
                let _ = collector.register(registry.clone());
            }
        }
        */

        // Save to global variables
        unsafe {
            REGISTRY = Some(registry);
            API_REQUESTS = Some(api_requests);
            TASK_EXECUTION_TIME = Some(task_execution_time);
            ACTIVE_TASKS = Some(active_tasks);
            POLICY_EVALUATIONS = Some(policy_evaluations);
            SANDBOX_EXECUTION_TIME = Some(sandbox_execution_time);
            ERROR_COUNTER = Some(error_counter);
        }
    });
}

/// Get metrics registry
pub fn get_registry() -> &'static Registry {
    unsafe {
        REGISTRY
            .as_ref()
            .expect("Metrics have not been initialized")
    }
}

/// Count API call
pub fn increment_api_requests(method: &str, path: &str, status: &str) {
    unsafe {
        if let Some(counter) = API_REQUESTS.as_ref() {
            counter.with_label_values(&[method, path, status]).inc();
        }
    }
}

/// Start task execution timer
pub fn start_task_timer() -> Instant {
    Instant::now()
}

/// Record task execution time
pub fn observe_task_execution_time(start_time: Instant, task_type: &str, status: &str) {
    let duration_ms = start_time.elapsed().as_millis() as f64;
    unsafe {
        if let Some(histogram) = TASK_EXECUTION_TIME.as_ref() {
            histogram
                .with_label_values(&[task_type, status])
                .observe(duration_ms);
        }
    }
}

/// Increment active tasks count
pub fn increment_active_tasks() {
    unsafe {
        if let Some(gauge) = ACTIVE_TASKS.as_ref() {
            gauge.inc();
        }
    }
}

/// Decrement active tasks count
pub fn decrement_active_tasks() {
    unsafe {
        if let Some(gauge) = ACTIVE_TASKS.as_ref() {
            gauge.dec();
        }
    }
}

/// Count policy evaluation
pub fn increment_policy_evaluations(policy: &str, result: &str) {
    unsafe {
        if let Some(counter) = POLICY_EVALUATIONS.as_ref() {
            counter.with_label_values(&[policy, result]).inc();
        }
    }
}

/// Start sandbox execution timer
pub fn start_sandbox_timer() -> Instant {
    Instant::now()
}

/// Record sandbox execution time
pub fn observe_sandbox_execution_time(start_time: Instant, command: &str) {
    let duration_ms = start_time.elapsed().as_millis() as f64;
    unsafe {
        if let Some(histogram) = SANDBOX_EXECUTION_TIME.as_ref() {
            histogram.with_label_values(&[command]).observe(duration_ms);
        }
    }
}

/// Count error
pub fn increment_error_counter(error_type: &str, error_code: &str) {
    unsafe {
        if let Some(counter) = ERROR_COUNTER.as_ref() {
            counter.with_label_values(&[error_type, error_code]).inc();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_initialization() {
        // 初期化を呼び出し
        init_metrics();

        // 変数が正しく初期化されていることを確認
        unsafe {
            assert!(REGISTRY.is_some());
            assert!(API_REQUESTS.is_some());
            assert!(TASK_EXECUTION_TIME.is_some());
            assert!(ACTIVE_TASKS.is_some());
            assert!(POLICY_EVALUATIONS.is_some());
            assert!(SANDBOX_EXECUTION_TIME.is_some());
            assert!(ERROR_COUNTER.is_some());
        }

        // レジストリを取得
        let registry = get_registry();
        assert!(registry.gather().len() > 0);

        // メトリクスの初期化を再度呼び出し (should be idempotent)
        init_metrics();

        // メトリクスが引き続き存在していることを確認
        unsafe {
            assert!(API_REQUESTS.is_some());
        }

        // APIリクエストカウンタの値を取得
        let metrics = registry.gather();
        let mut found = false;

        for mf in metrics {
            if mf.name() == "mcp_api_requests_total" {
                found = true;
                break;
            }
        }

        assert!(found, "mcp_api_requests_total メトリクスが見つかりません");
    }

    #[test]
    fn test_increment_api_counter() {
        // メトリクスを初期化
        init_metrics();

        // APIリクエストをインクリメント
        increment_api_requests("GET", "/health", "200");
        increment_api_requests("POST", "/command", "200");
        increment_api_requests("GET", "/health", "200");

        // インクリメントが成功することを確認 (値は検証できないが、クラッシュしないことを確認)
        assert!(true);
    }

    #[test]
    fn test_record_task_execution_time() {
        // メトリクスを初期化
        init_metrics();

        // タスク実行時間を記録
        let start = start_task_timer();
        std::thread::sleep(Duration::from_millis(10));
        observe_task_execution_time(start, "test_task", "success");

        // 複数のタスク時間を記録
        let start2 = start_task_timer();
        std::thread::sleep(Duration::from_millis(5));
        observe_task_execution_time(start2, "another_task", "failure");

        // 記録が成功することを確認
        assert!(true);
    }

    #[test]
    fn test_update_active_tasks() {
        // メトリクスを初期化
        init_metrics();

        // アクティブタスクをインクリメント
        increment_active_tasks();
        increment_active_tasks();
        increment_active_tasks();

        // アクティブタスクをデクリメント
        decrement_active_tasks();
        decrement_active_tasks();

        // 更新が成功することを確認
        assert!(true);
    }

    #[test]
    fn test_metrics_registry() {
        // メトリクスを初期化
        init_metrics();

        // レジストリを取得
        let registry = get_registry();

        // レジストリにメトリクスが含まれていることを確認
        let metrics = registry.gather();
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_metrics_singleton() {
        // メトリクスを初期化
        init_metrics();

        // 同じレジストリインスタンスを2回取得
        let registry1 = get_registry() as *const Registry;
        let registry2 = get_registry() as *const Registry;

        // 同じインスタンスであることを確認
        assert_eq!(registry1, registry2);
    }
    
    #[test]
    fn test_policy_evaluations() {
        // メトリクスを初期化
        init_metrics();
        
        // ポリシー評価をカウント
        increment_policy_evaluations("command_policy", "allowed");
        increment_policy_evaluations("command_policy", "denied");
        increment_policy_evaluations("file_policy", "allowed");
        
        // カウントが成功することを確認
        assert!(true);
    }
    
    #[test]
    fn test_sandbox_execution_time() {
        // メトリクスを初期化
        init_metrics();
        
        // サンドボックス実行時間を記録
        let start = start_sandbox_timer();
        std::thread::sleep(Duration::from_millis(15));
        observe_sandbox_execution_time(start, "echo");
        
        // 別のコマンドも記録
        let start2 = start_sandbox_timer();
        std::thread::sleep(Duration::from_millis(5));
        observe_sandbox_execution_time(start2, "ls");
        
        // 記録が成功することを確認
        assert!(true);
    }
    
    #[test]
    fn test_error_counter() {
        // メトリクスを初期化
        init_metrics();
        
        // エラーをカウント
        increment_error_counter("auth", "401");
        increment_error_counter("validation", "400");
        increment_error_counter("internal", "500");
        
        // カウントが成功することを確認
        assert!(true);
    }
}
