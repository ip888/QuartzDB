#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# QuartzDB Load Test Suite v2.0
# Performance & stress testing with real-time metrics
#═══════════════════════════════════════════════════════════════════════════════
# Usage: ./load_test.sh [--requests N] [--concurrency N] [--timeout SECONDS]
#═══════════════════════════════════════════════════════════════════════════════

set +e

#───────────────────────────────────────────────────────────────────────────────
# Configuration
#───────────────────────────────────────────────────────────────────────────────
BASE_URL="${BASE_URL:-http://localhost:8787}"
API_KEY="${API_KEY:-}"
REQUESTS="${REQUESTS:-100}"
CONCURRENCY="${CONCURRENCY:-5}"
TIMEOUT="${TIMEOUT:-10}"
TOTAL_START=$(date +%s%N)

while [[ $# -gt 0 ]]; do
    case $1 in
        --requests|-r)    REQUESTS="$2"; shift 2 ;;
        --concurrency|-c) CONCURRENCY="$2"; shift 2 ;;
        --timeout|-t)     TIMEOUT="$2"; shift 2 ;;
        *) shift ;;
    esac
done

#───────────────────────────────────────────────────────────────────────────────
# Colors
#───────────────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
DIM='\033[2m'
BOLD='\033[1m'
NC='\033[0m'

#───────────────────────────────────────────────────────────────────────────────
# Utilities
#───────────────────────────────────────────────────────────────────────────────
timestamp() { date +"%H:%M:%S"; }
elapsed_ms() { echo $(( ($1) / 1000000 )); }

log() { echo -e "${DIM}[$(timestamp)]${NC} ${BLUE}INFO${NC} $1"; }
ok() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
err() { echo -e "${RED}✗${NC} $1"; }

progress_bar() {
    local current=$1 total=$2 width=40
    local pct=$((current * 100 / total))
    local filled=$((pct * width / 100))
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=filled; i<width; i++)); do bar+="░"; done
    printf "\r  ${DIM}[${bar}]${NC} %3d%% (%d/%d)" "$pct" "$current" "$total"
}

curl_cmd() {
    local args=(-s --connect-timeout "$TIMEOUT" --max-time "$TIMEOUT" -w "%{http_code}")
    [[ -n "$API_KEY" ]] && args+=(-H "X-API-Key: $API_KEY")
    curl "${args[@]}" "$@"
}

generate_vector() {
    local dim=${1:-384}
    if command -v python3 &>/dev/null; then
        python3 -c "import random; print('[' + ','.join(f'{random.random():.6f}' for _ in range($dim)) + ']')"
    else
        local vec="["
        for ((i=1; i<=dim; i++)); do
            vec+="0.$((RANDOM % 1000000))"
            [[ $i -lt $dim ]] && vec+=","
        done
        echo "${vec}]"
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Test 1: Sequential Insert Performance
#───────────────────────────────────────────────────────────────────────────────
test_sequential_insert() {
    local count=$1
    local success=0 failed=0
    local latencies=()
    local start=$(date +%s%N)
    
    log "Test 1: Sequential Insert Performance"
    log "Inserting $count vectors sequentially..."
    echo ""
    
    for i in $(seq 1 $count); do
        progress_bar $i $count
        
        local vec_start=$(date +%s%N)
        local vector
        vector=$(generate_vector 384)
        local http_code
        http_code=$(curl_cmd -o /dev/null -X POST "$BASE_URL/api/vector/insert" \
            -H "Content-Type: application/json" \
            -d "{\"id\": \"load_seq_$i\", \"vector\": $vector, \"metadata\": {\"i\": $i}}" 2>/dev/null)
        local vec_end=$(date +%s%N)
        
        local latency=$(elapsed_ms $((vec_end - vec_start)))
        latencies+=($latency)
        
        if [[ "$http_code" == "200" ]]; then
            ((success++))
        else
            ((failed++))
        fi
    done
    echo ""
    
    local end=$(date +%s%N)
    local total_time=$(elapsed_ms $((end - start)))
    local rate=$((count * 1000 / (total_time + 1)))
    
    # Calculate latency stats
    local sum=0 min=999999 max=0
    for lat in "${latencies[@]}"; do
        sum=$((sum + lat))
        [[ $lat -lt $min ]] && min=$lat
        [[ $lat -gt $max ]] && max=$lat
    done
    local avg=$((sum / ${#latencies[@]}))
    
    echo ""
    echo -e "  ${BOLD}Results:${NC}"
    echo -e "    Requests:     $count"
    echo -e "    Success:      ${GREEN}$success${NC}"
    echo -e "    Failed:       ${RED}$failed${NC}"
    echo -e "    Total Time:   ${total_time}ms"
    echo -e "    Throughput:   ${rate} req/sec"
    echo -e "    Latency:"
    echo -e "      Min:        ${min}ms"
    echo -e "      Avg:        ${avg}ms"
    echo -e "      Max:        ${max}ms"
    
    [[ $success -ge $((count * 8 / 10)) ]]
}

#───────────────────────────────────────────────────────────────────────────────
# Test 2: Search Latency Distribution
#───────────────────────────────────────────────────────────────────────────────
test_search_latency() {
    local count=$1
    local latencies=()
    local start=$(date +%s%N)
    
    echo ""
    log "Test 2: Search Latency Distribution"
    log "Executing $count searches..."
    echo ""
    
    local vector
    vector=$(generate_vector 384)
    
    for i in $(seq 1 $count); do
        progress_bar $i $count
        
        local search_start=$(date +%s%N)
        curl_cmd -o /dev/null -X POST "$BASE_URL/api/vector/search" \
            -H "Content-Type: application/json" \
            -d "{\"vector\": $vector, \"k\": 10}" 2>/dev/null
        local search_end=$(date +%s%N)
        
        local latency=$(elapsed_ms $((search_end - search_start)))
        latencies+=($latency)
    done
    echo ""
    
    # Sort latencies for percentiles
    IFS=$'\n' sorted=($(sort -n <<<"${latencies[*]}")); unset IFS
    
    local p50_idx=$((count / 2))
    local p95_idx=$((count * 95 / 100))
    local p99_idx=$((count * 99 / 100))
    
    local sum=0 min=999999 max=0
    for lat in "${latencies[@]}"; do
        sum=$((sum + lat))
        [[ $lat -lt $min ]] && min=$lat
        [[ $lat -gt $max ]] && max=$lat
    done
    local avg=$((sum / ${#latencies[@]}))
    
    echo ""
    echo -e "  ${BOLD}Latency Distribution:${NC}"
    echo -e "    Min:          ${min}ms"
    echo -e "    Avg:          ${avg}ms"
    echo -e "    P50:          ${sorted[$p50_idx]}ms"
    echo -e "    P95:          ${sorted[$p95_idx]}ms"
    echo -e "    P99:          ${sorted[$p99_idx]}ms"
    echo -e "    Max:          ${max}ms"
    
    [[ $avg -lt 500 ]]
}

#───────────────────────────────────────────────────────────────────────────────
# Test 3: Concurrent Users Simulation
#───────────────────────────────────────────────────────────────────────────────
test_concurrent_users() {
    local users=$1
    local requests_per_user=10
    local total=$((users * requests_per_user))
    local pids=()
    local results_dir="/tmp/quartz_load_$$"
    mkdir -p "$results_dir"
    
    echo ""
    log "Test 3: Concurrent Users Simulation"
    log "Simulating $users concurrent users, $requests_per_user requests each..."
    echo ""
    
    local start=$(date +%s%N)
    
    # Spawn concurrent workers
    for u in $(seq 1 $users); do
        (
            local success=0 failed=0
            for r in $(seq 1 $requests_per_user); do
                local vector
                vector=$(generate_vector 384)
                local http_code
                http_code=$(curl_cmd -o /dev/null -X POST "$BASE_URL/api/vector/insert" \
                    -H "Content-Type: application/json" \
                    -d "{\"id\": \"load_u${u}_r${r}\", \"vector\": $vector, \"metadata\": {}}" 2>/dev/null)
                if [[ "$http_code" == "200" ]]; then ((success++)); else ((failed++)); fi
            done
            echo "$success $failed" > "$results_dir/user_$u.txt"
        ) &
        pids+=($!)
    done
    
    # Progress while waiting
    local completed=0
    while [[ $completed -lt $users ]]; do
        completed=$(ls "$results_dir" 2>/dev/null | wc -l)
        progress_bar $completed $users
        sleep 0.5
    done
    echo ""
    
    # Wait for all
    for pid in "${pids[@]}"; do
        wait $pid 2>/dev/null
    done
    
    local end=$(date +%s%N)
    local total_time=$(elapsed_ms $((end - start)))
    
    # Aggregate results
    local total_success=0 total_failed=0
    for file in "$results_dir"/user_*.txt; do
        read s f < "$file"
        total_success=$((total_success + s))
        total_failed=$((total_failed + f))
    done
    rm -rf "$results_dir"
    
    local rate=$((total * 1000 / (total_time + 1)))
    local error_rate=$((total_failed * 100 / total))
    
    echo ""
    echo -e "  ${BOLD}Results:${NC}"
    echo -e "    Users:        $users concurrent"
    echo -e "    Total Req:    $total"
    echo -e "    Success:      ${GREEN}$total_success${NC}"
    echo -e "    Failed:       ${RED}$total_failed${NC}"
    echo -e "    Error Rate:   ${error_rate}%"
    echo -e "    Total Time:   ${total_time}ms"
    echo -e "    Throughput:   ${rate} req/sec"
    
    [[ $error_rate -le 25 ]]  # Allow up to 25% errors for concurrent load
}

#───────────────────────────────────────────────────────────────────────────────
# Test 4: Memory Pressure (Large Batch)
#───────────────────────────────────────────────────────────────────────────────
test_memory_pressure() {
    local count=50
    local success=0 failed=0
    
    echo ""
    log "Test 4: Memory Pressure Test"
    log "Inserting $count vectors in rapid succession..."
    echo ""
    
    local start=$(date +%s%N)
    
    for i in $(seq 1 $count); do
        progress_bar $i $count
        
        local vector
        vector=$(generate_vector 384)
        local http_code
        http_code=$(curl_cmd -o /dev/null -X POST "$BASE_URL/api/vector/insert" \
            -H "Content-Type: application/json" \
            -d "{\"id\": \"load_mem_$i\", \"vector\": $vector, \"metadata\": {}}" 2>/dev/null)
        
        if [[ "$http_code" == "200" ]]; then ((success++)); else ((failed++)); fi
    done
    echo ""
    
    local end=$(date +%s%N)
    local total_time=$(elapsed_ms $((end - start)))
    
    echo ""
    echo -e "  ${BOLD}Results:${NC}"
    echo -e "    Vectors:      $count"
    echo -e "    Success:      ${GREEN}$success${NC}"
    echo -e "    Failed:       ${RED}$failed${NC}"
    echo -e "    Total Time:   ${total_time}ms"
    
    [[ $success -ge $((count * 8 / 10)) ]]
}

#───────────────────────────────────────────────────────────────────────────────
# Test 5: Sustained Load
#───────────────────────────────────────────────────────────────────────────────
test_sustained_load() {
    local duration=10  # seconds
    local success=0 failed=0 total=0
    
    echo ""
    log "Test 5: Sustained Load Test"
    log "Running continuous requests for ${duration}s..."
    echo ""
    
    local start=$(date +%s)
    local end=$((start + duration))
    
    while [[ $(date +%s) -lt $end ]]; do
        local elapsed=$(($(date +%s) - start))
        local pct=$((elapsed * 100 / duration))
        printf "\r  ${DIM}[%3d%%]${NC} Elapsed: %ds, Requests: %d, Success: %d, Failed: %d" \
            "$pct" "$elapsed" "$total" "$success" "$failed"
        
        local vector
        vector=$(generate_vector 384)
        local http_code
        http_code=$(curl_cmd -o /dev/null -X POST "$BASE_URL/api/vector/search" \
            -H "Content-Type: application/json" \
            -d "{\"vector\": $vector, \"k\": 10}" 2>/dev/null)
        
        ((total++))
        if [[ "$http_code" == "200" ]]; then ((success++)); else ((failed++)); fi
    done
    echo ""
    
    local rate=$((total / duration))
    local error_rate=$((failed * 100 / (total + 1)))
    
    echo ""
    echo -e "  ${BOLD}Results:${NC}"
    echo -e "    Duration:     ${duration}s"
    echo -e "    Total Req:    $total"
    echo -e "    Success:      ${GREEN}$success${NC}"
    echo -e "    Failed:       ${RED}$failed${NC}"
    echo -e "    Error Rate:   ${error_rate}%"
    echo -e "    Throughput:   ${rate} req/sec"
    
    [[ $error_rate -lt 10 ]]
}

#═══════════════════════════════════════════════════════════════════════════════
# Main
#═══════════════════════════════════════════════════════════════════════════════
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║              QuartzDB Load Test Suite v2.0                        ║"
    echo "║           Performance & Stress Testing                            ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${BOLD}Configuration${NC}"
    echo -e "  Server:      $BASE_URL"
    echo -e "  Requests:    $REQUESTS"
    echo -e "  Concurrency: $CONCURRENCY"
    echo -e "  Timeout:     ${TIMEOUT}s"
    echo -e "  API Key:     ${API_KEY:+${API_KEY:0:8}...}${API_KEY:-none}"
    echo ""
    
    log "Checking server connectivity..."
    if ! curl_cmd -o /dev/null "$BASE_URL/health" 2>/dev/null; then
        err "Server not reachable at $BASE_URL"
        exit 1
    fi
    ok "Server is online"
    
    local passed=0 failed=0
    declare -a results=()
    
    # Run tests
    local tests=(
        "test_sequential_insert:$((REQUESTS / 5)):Sequential Insert"
        "test_search_latency:$((REQUESTS / 5)):Search Latency"
        "test_concurrent_users:$CONCURRENCY:Concurrent Users"
        "test_memory_pressure::Memory Pressure"
        "test_sustained_load::Sustained Load"
    )
    
    for test_spec in "${tests[@]}"; do
        IFS=':' read -r func arg name <<< "$test_spec"
        local start=$(date +%s%N)
        
        if $func $arg; then
            ((passed++))
            local end=$(date +%s%N)
            local duration=$(elapsed_ms $((end - start)))
            results+=("PASS|$name|$duration")
            echo ""
            ok "$name completed"
        else
            ((failed++))
            local end=$(date +%s%N)
            local duration=$(elapsed_ms $((end - start)))
            results+=("FAIL|$name|$duration")
            echo ""
            err "$name failed"
        fi
    done
    
    local total_end=$(date +%s%N)
    local total_duration=$(elapsed_ms $((total_end - TOTAL_START)))
    
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║                    Load Test Summary                              ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    printf "  %-25s %-8s %s\n" "TEST" "STATUS" "TIME"
    printf "  %-25s %-8s %s\n" "─────────────────────────" "──────" "────────"
    for result in "${results[@]}"; do
        IFS='|' read -r status name time <<< "$result"
        local color=$([[ "$status" == "PASS" ]] && echo "$GREEN" || echo "$RED")
        printf "  %-25s ${color}%-8s${NC} %sms\n" "$name" "$status" "$time"
    done
    echo ""
    echo -e "  ${BOLD}Total:${NC}    $((passed + failed)) tests"
    echo -e "  ${GREEN}Passed:${NC}   $passed"
    echo -e "  ${RED}Failed:${NC}   $failed"
    echo -e "  ${BOLD}Duration:${NC} ${total_duration}ms"
    echo ""
    
    if [[ $failed -eq 0 ]]; then
        echo -e "  ${GREEN}${BOLD}✓ All load tests passed!${NC}"
        exit 0
    else
        echo -e "  ${RED}${BOLD}✗ $failed test(s) failed${NC}"
        exit 1
    fi
}

main "$@"
