#!/usr/bin/env python3
"""
MCP Security Gateway - LSコマンド実行フローE2Eテスト

このスクリプトは、MCP Security Gatewayが正しくlsコマンドを実行できることを検証します。
gRPCサーバーを起動し、コマンド実行し、結果を検証します。

前提条件:
- mcp-gatewayがビルドされていること
- grpcurlがインストールされていること
- pexpectがインストールされていること
- Pythonがインストールされていること (3.7以上)

使用方法:
    python ls_flow_test.py
"""

import os
import sys
import time
import json
import tempfile
import subprocess
import unittest
import pexpect
import signal
import uuid
from pathlib import Path


class LSFlowTest(unittest.TestCase):
    """MCPセキュリティゲートウェイのlsコマンド実行フローのE2Eテスト"""

    def setUp(self):
        """テスト環境のセットアップ"""
        # 一時ディレクトリを作成
        self.temp_dir = tempfile.TemporaryDirectory()
        self.workspace_path = self.temp_dir.name

        # テスト用ファイルを作成
        test_files = ['file1.txt', 'file2.txt', 'file3.txt']
        for file_name in test_files:
            with open(os.path.join(self.workspace_path, file_name), 'w') as f:
                f.write(f'Content of {file_name}')

        # サービスの起動
        self.server_process = pexpect.spawn(
            'cargo run --bin mcp-gateway -- serve --host 127.0.0.1 --port 50051',
            timeout=10
        )
        # 'Server listening'が出力されるまで待機
        self.server_process.expect('Server listening', timeout=15)
        
        # サーバーが起動するまで少し待機
        time.sleep(2)

    def tearDown(self):
        """テスト環境のクリーンアップ"""
        # サーバープロセスの終了
        if hasattr(self, 'server_process') and self.server_process.isalive():
            # SIGTERM送信
            self.server_process.kill(signal.SIGTERM)
            self.server_process.wait()

        # 一時ディレクトリの削除
        if hasattr(self, 'temp_dir'):
            self.temp_dir.cleanup()

    def test_ls_command_execution(self):
        """lsコマンドの実行フロー全体をテスト"""
        # 1. ヘルスチェック
        health_result = self._run_grpcurl("mcp.McpService/Health", "{}")
        self.assertEqual(health_result.get('status'), 'healthy')
        
        # 2. lsコマンドを実行
        command_request = {
            "command": "ls",
            "args": ["-la", self.workspace_path],
            "timeout": 30
        }
        
        ls_result = self._run_grpcurl(
            "mcp.McpService/ExecuteCommand", 
            json.dumps(command_request)
        )
        
        # タスクIDを取得
        task_id = ls_result.get('task_id')
        self.assertIsNotNone(task_id)
        
        # 3. タスクステータスをチェック (最大10秒待機)
        status_result = None
        for _ in range(10):
            status_result = self._run_grpcurl(
                "mcp.McpService/GetTaskStatus",
                json.dumps({"task_id": task_id})
            )
            
            if status_result.get('task_info', {}).get('status') in ['TASK_COMPLETED', 'TASK_FAILED']:
                break
                
            time.sleep(1)
        
        # 4. 結果の検証
        self.assertIsNotNone(status_result)
        self.assertEqual(status_result.get('task_info', {}).get('status'), 'TASK_COMPLETED')
        
        # 結果に3つのテストファイルが含まれていることを確認
        stdout = status_result.get('result', {}).get('stdout', '')
        for file_name in ['file1.txt', 'file2.txt', 'file3.txt']:
            self.assertIn(file_name, stdout)
        
        # 5. メトリクスの検証
        metrics_output = subprocess.check_output(
            ['curl', '-s', 'http://localhost:9090/metrics'],
            universal_newlines=True
        )
        
        # タスク実行時間メトリクスが存在することを確認
        self.assertIn('mcp_task_latency_ms', metrics_output)
        
        # API呼び出し数メトリクスが存在することを確認
        self.assertIn('mcp_api_requests_total', metrics_output)

    def _run_grpcurl(self, method, request_json):
        """gRPCurlを使用してgRPCメソッドを呼び出す"""
        cmd = [
            'grpcurl',
            '-plaintext',
            '-d', request_json,
            '127.0.0.1:50051',
            method
        ]
        
        try:
            result = subprocess.check_output(cmd, universal_newlines=True)
            return json.loads(result)
        except subprocess.CalledProcessError as e:
            print(f"Error executing grpcurl: {e}")
            print(f"Stdout: {e.stdout if hasattr(e, 'stdout') else 'N/A'}")
            print(f"Stderr: {e.stderr if hasattr(e, 'stderr') else 'N/A'}")
            raise


if __name__ == '__main__':
    unittest.main() 