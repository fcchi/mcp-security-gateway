#!/usr/bin/env python3
"""
MCP Security Gateway - Performance Test

Locustを使用した負荷テストスクリプト。
MCPセキュリティゲートウェイの性能ベースラインを検証するために使用します。

前提条件:
- locustがインストールされていること (`pip install locust`)
- MCP Security Gatewayが実行中であること

使用方法:
    locust -f locustfile.py --host=http://localhost:8081

または：
    python -m locust -f locustfile.py --host=http://localhost:8081
"""

import json
import time
import uuid
import random
from locust import HttpUser, TaskSet, task, constant_throughput, events, stats
from locust.runners import MasterRunner, LocalRunner

# 基本設定
DEFAULT_TIMEOUT = 30
RPS_TARGET = 100
DEFAULT_DURATION = 300  # 5分

# 結果格納
p95_stats = []

# テスト用コマンド
COMMANDS = [
    {"command": "ls", "args": ["-la", "/workspace"]},
    {"command": "cat", "args": ["/etc/passwd"]},
    {"command": "find", "args": ["/workspace", "-type", "f", "-name", "*.txt"]},
    {"command": "echo", "args": ["Hello, World!"]},
    {"command": "grep", "args": ["-i", "root", "/etc/passwd"]}
]

class MCPGatewayTasks(TaskSet):
    """MCPセキュリティゲートウェイへのタスク"""
    
    def on_start(self):
        """初期化処理"""
        # 認証トークンを設定するなどの前処理がある場合はここで実行
        self.headers = {"Content-Type": "application/json"}
        self.task_ids = []
    
    @task(10)
    def health_check(self):
        """ヘルスチェックAPI"""
        response = self.client.get("/health", 
                                   headers=self.headers,
                                   name="Health Check")
        
        if response.status_code != 200:
            response.failure(f"Health check failed: {response.text}")
    
    @task(60)
    def execute_command(self):
        """コマンド実行API"""
        # ランダムなコマンドを選択
        command_data = random.choice(COMMANDS)
        
        payload = {
            "command": command_data["command"],
            "args": command_data["args"],
            "timeout": DEFAULT_TIMEOUT
        }
        
        # コマンド実行リクエスト
        start_time = time.time()
        response = self.client.post("/v1/execute", 
                                    json=payload,
                                    headers=self.headers,
                                    name=f"Execute {command_data['command']}")
        
        if response.status_code != 200:
            response.failure(f"Command execution failed: {response.text}")
            return
        
        # タスクIDを取得
        try:
            result = response.json()
            task_id = result.get("task_id")
            if not task_id:
                response.failure("No task_id in response")
                return
                
            self.task_ids.append(task_id)
            
            # タスク完了を待機
            self.wait_for_task_completion(task_id, start_time)
            
        except json.JSONDecodeError:
            response.failure("Invalid JSON response")
    
    @task(30)
    def get_task_status(self):
        """タスクステータス取得API"""
        if not self.task_ids:
            return
            
        # 過去のタスクIDからランダムに選択
        task_id = random.choice(self.task_ids)
        
        response = self.client.get(f"/v1/tasks/{task_id}", 
                                   headers=self.headers,
                                   name="Get Task Status")
        
        if response.status_code != 200:
            response.failure(f"Get task status failed: {response.text}")
    
    def wait_for_task_completion(self, task_id, start_time, max_wait=10):
        """タスクの完了を待機する"""
        elapsed_time = 0
        while elapsed_time < max_wait:
            response = self.client.get(f"/v1/tasks/{task_id}", 
                                       headers=self.headers,
                                       name="Poll Task Status")
                                       
            if response.status_code != 200:
                response.failure(f"Polling task failed: {response.text}")
                return
                
            try:
                result = response.json()
                status = result.get("status")
                
                if status in ["COMPLETED", "FAILED", "ERROR"]:
                    # 全体の実行時間を記録
                    total_time = time.time() - start_time
                    # メトリクス記録するなど必要な処理をここに追加
                    return
                    
            except json.JSONDecodeError:
                response.failure("Invalid JSON response while polling")
                return
                
            time.sleep(0.5)
            elapsed_time += 0.5

class MCPUser(HttpUser):
    """MCPゲートウェイを呼び出すユーザー"""
    tasks = [MCPGatewayTasks]
    # 100 RPSを目標とするために1ユーザーあたりの待機時間を調整
    # locustは--usersの数と組み合わせてRPSを制御する
    wait_time = constant_throughput(5)  # 1ユーザーあたり5RPS、20ユーザーで100RPSになる

@events.test_start.add_listener
def on_test_start(environment, **kwargs):
    """テスト開始時の処理"""
    print(f"--- Starting performance test: target {RPS_TARGET} RPS for {DEFAULT_DURATION}s ---")
    # グローバル統計の設定
    stats.PERCENTILES_TO_RECORD = [0.50, 0.90, 0.95, 0.99]

@events.test_stop.add_listener
def on_test_stop(environment, **kwargs):
    """テスト終了時の処理"""
    print("--- Performance test completed ---")
    
    # 結果のサマリーを表示
    if isinstance(environment.runner, (LocalRunner, MasterRunner)):
        stats_printer = environment.stats.console_writer
        
        # p95レイテンシを評価
        for name, request_stats in environment.stats.entries.items():
            if request_stats.num_requests == 0:
                continue
                
            print(f"Endpoint: {name}")
            print(f"  Requests: {request_stats.num_requests}")
            print(f"  50%ile: {request_stats.get_percentile(0.5):.2f}ms")
            print(f"  90%ile: {request_stats.get_percentile(0.9):.2f}ms")
            print(f"  95%ile: {request_stats.get_percentile(0.95):.2f}ms")
            print(f"  99%ile: {request_stats.get_percentile(0.99):.2f}ms")
            print(f"  RPS: {request_stats.total_rps}")
            
            # SLO評価
            p95 = request_stats.get_percentile(0.95)
            if p95 > 400:  # p95 < 400ms目標
                print(f"  WARNING: 95%ile latency {p95:.2f}ms exceeds SLO target of 400ms")
            else:
                print(f"  SUCCESS: 95%ile latency {p95:.2f}ms within SLO target of 400ms")

# コマンドライン実行時のヘルプ
if __name__ == "__main__":
    print(__doc__)
    print("\nRun with: locust -f locustfile.py --host=http://localhost:8081") 