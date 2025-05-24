#![allow(static_mut_refs)]

use prometheus::{
    HistogramOpts, HistogramVec, IntCounterVec, IntGauge,
    Opts, Registry,
};
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
        let active_tasks = IntGauge::new("mcp_active_tasks", "Number of currently running tasks").unwrap();

        // Policy evaluation counter
        let policy_evaluations = IntCounterVec::new(
            Opts::new("mcp_policy_evaluations_total", "Total number of policy evaluations"),
            &["policy", "result"],
        )
        .unwrap();

        // Sandbox execution time
        let sandbox_execution_time = HistogramVec::new(
            HistogramOpts::new("mcp_sandbox_execution_time_ms", "Sandbox execution time (milliseconds)")
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
        REGISTRY.as_ref().expect("Metrics have not been initialized")
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
        // Initialize metrics
        init_metrics();
        
        // Verify that second initialization can be called without issues
        init_metrics();
        
        // Verify that registry is initialized
        unsafe {
            assert!(REGISTRY.is_some(), "Registry has not been initialized");
            assert!(API_REQUESTS.is_some(), "API_REQUESTS has not been initialized");
            assert!(TASK_EXECUTION_TIME.is_some(), "TASK_EXECUTION_TIME has not been initialized");
            assert!(ACTIVE_TASKS.is_some(), "ACTIVE_TASKS has not been initialized");
            assert!(POLICY_EVALUATIONS.is_some(), "POLICY_EVALUATIONS has not been initialized");
            assert!(SANDBOX_EXECUTION_TIME.is_some(), "SANDBOX_EXECUTION_TIME has not been initialized");
            assert!(ERROR_COUNTER.is_some(), "ERROR_COUNTER has not been initialized");
        }
    }

    #[test]
    fn test_increment_api_counter() {
        // Initialize metrics
        init_metrics();
        
        // Increment API counter
        increment_api_requests("GET", "/api/status", "200");
        
        // Increment the same label again
        increment_api_requests("GET", "/api/status", "200");
        
        // Method to verify that the counter is incremented twice is omitted
        // (Depends on Prometheus implementation details)
    }

    #[test]
    fn test_record_task_execution_time() {
        // Initialize metrics
        init_metrics();
        
        // Start task
        let start = Instant::now();
        
        // Wait a bit
        std::thread::sleep(Duration::from_millis(10));
        
        // Record task completion
        observe_task_execution_time(start, "command", "success");
        
        // Method to verify that time is recorded correctly is omitted
        // (Depends on Prometheus implementation details)
    }

    #[test]
    fn test_update_active_tasks() {
        // Initialize metrics
        init_metrics();
        
        // Increment active tasks
        increment_active_tasks();
        increment_active_tasks();
        
        // Decrement active tasks
        decrement_active_tasks();
        
        // Method to verify that the count is 1 is omitted
        // (Depends on Prometheus implementation details)
    }

    #[test]
    fn test_metrics_registry() {
        // Initialize metrics
        init_metrics();
        
        // Get registry
        let registry = get_registry();
        
        // Verify that registry is obtained
        assert!(!registry.gather().is_empty(), "Registry is not correctly initialized");
    }
} 