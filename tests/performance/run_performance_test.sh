#!/bin/bash
# MCP Security Gateway - Performance Test Runner
# 
# Locustを使用して100 RPS x 5分のベースラインテストを実行します。
# テスト結果をCSVファイルとHTML形式でレポートします。
#
# 前提条件:
# - locustがインストールされていること (`pip install locust`)
# - MCP Security Gatewayが実行中であること
#
# 使用方法:
#     ./run_performance_test.sh [host] [users] [spawn_rate] [duration]
#
# パラメータ:
#     host:        ターゲットホスト (デフォルト: http://localhost:8081)
#     users:       同時ユーザー数 (デフォルト: 20)
#     spawn_rate:  毎秒あたりの新規ユーザー数 (デフォルト: 5)
#     duration:    テスト継続時間（秒） (デフォルト: 300 = 5分)

set -e

# パラメータ設定
HOST=${1:-"http://localhost:8081"}
USERS=${2:-20}
SPAWN_RATE=${3:-5}
DURATION=${4:-300}
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_DIR="performance_reports/${TIMESTAMP}"

# 必要なディレクトリを作成
mkdir -p "${REPORT_DIR}"

echo "======================================================"
echo "MCP Security Gateway - Performance Test"
echo "======================================================"
echo "Target Host:     ${HOST}"
echo "Users:           ${USERS}"
echo "Spawn Rate:      ${SPAWN_RATE}/sec"
echo "Duration:        ${DURATION} seconds"
echo "Report Location: ${REPORT_DIR}"
echo "======================================================"

# locustが利用可能かチェック
if ! command -v locust &> /dev/null; then
    echo "Error: locust command not found. Please install with: pip install locust"
    exit 1
fi

# ホストが到達可能かチェック
if ! curl -s --head --fail "${HOST}/health" &> /dev/null; then
    echo "Error: Host ${HOST} is not reachable or health check failed."
    echo "Make sure MCP Security Gateway is running and accessible."
    exit 1
fi

echo "[$(date +"%H:%M:%S")] Starting performance test..."

# Locustをヘッドレスモードで実行
locust -f tests/performance/locustfile.py \
  --host="${HOST}" \
  --users="${USERS}" \
  --spawn-rate="${SPAWN_RATE}" \
  --run-time="${DURATION}s" \
  --headless \
  --csv="${REPORT_DIR}/stats" \
  --html="${REPORT_DIR}/report.html"

echo "[$(date +"%H:%M:%S")] Performance test completed."
echo "Results saved to: ${REPORT_DIR}"

# レポートの分析
echo "======================================================"
echo "Performance Summary"
echo "======================================================"

# CSVから95パーセンタイルのデータを抽出して表示
if [ -f "${REPORT_DIR}/stats_stats.csv" ]; then
    echo "95th Percentile Response Times (ms):"
    tail -n +2 "${REPORT_DIR}/stats_stats.csv" | \
    awk -F ',' '{print $1 ": " $8 " ms"}' | sort
    
    # SLO違反チェック
    if tail -n +2 "${REPORT_DIR}/stats_stats.csv" | \
       awk -F ',' '{if ($8 > 400) print $1}' | grep -q .; then
        echo ""
        echo "WARNING: SLO違反あり - 以下のエンドポイントでp95 > 400ms"
        tail -n +2 "${REPORT_DIR}/stats_stats.csv" | \
        awk -F ',' '{if ($8 > 400) print "  " $1 ": " $8 " ms"}' | sort
        echo ""
        echo "対応が必要です！"
    else
        echo ""
        echo "SUCCESS: すべてのエンドポイントでSLO目標 (p95 <= 400ms) を達成しました。"
    fi
fi

echo ""
echo "詳細レポート: ${REPORT_DIR}/report.html"
echo "======================================================" 