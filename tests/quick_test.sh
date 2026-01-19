#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# QuartzDB Quick Test Suite v2.0
# Production-grade API validation with metrics and progress tracking
#═══════════════════════════════════════════════════════════════════════════════
# Usage: ./quick_test.sh [--verbose] [--timeout SECONDS]
# Environment: BASE_URL, API_KEY
#═══════════════════════════════════════════════════════════════════════════════

set +e

#───────────────────────────────────────────────────────────────────────────────
# Configuration
#───────────────────────────────────────────────────────────────────────────────
BASE_URL="${BASE_URL:-http://localhost:8787}"
API_KEY="${API_KEY:-}"
VERBOSE=false
TIMEOUT=10
TOTAL_START=$(date +%s%N)

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v) VERBOSE=true; shift ;;
        --timeout|-t) TIMEOUT="$2"; shift 2 ;;
        *) shift ;;
    esac
done

#───────────────────────────────────────────────────────────────────────────────
# Colors & Formatting
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
# Metrics
#───────────────────────────────────────────────────────────────────────────────
PASSED=0
FAILED=0
TOTAL=0
declare -a TEST_RESULTS=()
declare -a TEST_TIMES=()

#───────────────────────────────────────────────────────────────────────────────
# Utilities
#───────────────────────────────────────────────────────────────────────────────
timestamp() { date +"%H:%M:%S"; }
elapsed_ms() { echo $(( ($1) / 1000000 )); }

log_info()  { echo -e "${DIM}[$(timestamp)]${NC} ${BLUE}INFO${NC}  $1"; }
log_pass()  { echo -e "${DIM}[$(timestamp)]${NC} ${GREEN}PASS${NC}  $1"; }
log_fail()  { echo -e "${DIM}[$(timestamp)]${NC} ${RED}FAIL${NC}  $1"; }
log_debug() { [[ $VERBOSE == true ]] && echo -e "${DIM}[$(timestamp)] DEBUG $1${NC}"; }

progress() {
    local current=$1 total=$2 desc=$3
    local pct=$((current * 100 / total))
    local filled=$((pct / 5))
    local empty=$((20 - filled))
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=0; i<empty; i++)); do bar+="░"; done
    printf "\r  ${DIM}[${bar}]${NC} %3d%% %s" "$pct" "$desc"
}

curl_cmd() {
    local args=(-s --connect-timeout "$TIMEOUT" --max-time "$TIMEOUT")
    [[ -n "$API_KEY" ]] && args+=(-H "X-API-Key: $API_KEY")
    curl "${args[@]}" "$@"
}

# Fast vector generation (precomputed for speed)
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
# Test Framework
#───────────────────────────────────────────────────────────────────────────────
run_test() {
    local test_name="$1"
    local test_desc="$2"
    local test_func="$3"
    
    ((TOTAL++))
    local start=$(date +%s%N)
    
    echo ""
    echo -e "${BOLD}━━━ Test $TOTAL: $test_name ━━━${NC}"
    echo -e "${DIM}    $test_desc${NC}"
    
    local result duration
    if $test_func; then
        result="PASS"
        ((PASSED++))
    else
        result="FAIL"
        ((FAILED++))
    fi
    
    local end=$(date +%s%N)
    duration=$(elapsed_ms $((end - start)))
    
    if [[ "$result" == "PASS" ]]; then
        log_pass "$test_name ${DIM}(${duration}ms)${NC}"
    else
        log_fail "$test_name ${DIM}(${duration}ms)${NC}"
    fi
    
    TEST_RESULTS+=("$result|$test_name|$duration")
    TEST_TIMES+=("$duration")
}

#───────────────────────────────────────────────────────────────────────────────
# Tests
#───────────────────────────────────────────────────────────────────────────────
test_health() {
    log_info "Checking health endpoint..."
    local response
    response=$(curl_cmd "$BASE_URL/health" 2>&1)
    log_debug "Response: $response"
    
    if echo "$response" | grep -qi "healthy\|ok\|success"; then
        log_info "Server status: healthy"
        return 0
    fi
    return 1
}

test_insert() {
    log_info "Generating 384-dim vector..."
    local vector id
    vector=$(generate_vector 384)
    id="quick_test_$(date +%s)"
    
    log_info "Inserting vector: $id"
    local response
    response=$(curl_cmd -X POST "$BASE_URL/api/vector/insert" \
        -H "Content-Type: application/json" \
        -d "{\"id\": \"$id\", \"vector\": $vector, \"metadata\": {\"test\": true}}" 2>&1)
    log_debug "Response: $response"
    
    if echo "$response" | jq -e '.success == true' &>/dev/null; then
        log_info "Insert successful"
        return 0
    fi
    log_info "Insert failed: ${response:0:80}"
    return 1
}

test_search() {
    log_info "Generating query vector..."
    local vector
    vector=$(generate_vector 384)
    
    log_info "Executing k-NN search (k=5)..."
    local response
    response=$(curl_cmd -X POST "$BASE_URL/api/vector/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\": $vector, \"k\": 5}" 2>&1)
    log_debug "Response: $response"
    
    if echo "$response" | jq -e '.success == true' &>/dev/null; then
        local count
        count=$(echo "$response" | jq '.results | length')
        log_info "Found $count results"
        return 0
    fi
    return 1
}

test_delete() {
    local id="delete_test_$(date +%s)"
    local vector
    vector=$(generate_vector 384)
    
    log_info "Creating vector to delete: $id"
    curl_cmd -X POST "$BASE_URL/api/vector/insert" \
        -H "Content-Type: application/json" \
        -d "{\"id\": \"$id\", \"vector\": $vector, \"metadata\": {}}" &>/dev/null
    
    log_info "Deleting vector..."
    local response
    response=$(curl_cmd -X DELETE "$BASE_URL/api/vector/delete/$id" 2>&1)
    log_debug "Response: $response"
    
    if echo "$response" | grep -qi "success\|deleted"; then
        log_info "Delete successful"
        return 0
    fi
    return 1
}

test_stats() {
    log_info "Fetching statistics..."
    local response
    response=$(curl_cmd "$BASE_URL/api/vector/stats" 2>&1)
    log_debug "Response: $response"
    
    # Check for both possible field names
    if echo "$response" | jq -e '.total_vectors >= 0 or .num_vectors >= 0' &>/dev/null; then
        local total active
        total=$(echo "$response" | jq '.total_vectors // .num_vectors')
        active=$(echo "$response" | jq '.active_vectors // .num_active // .total_vectors // .num_vectors')
        log_info "Index: $total total, $active active vectors"
        return 0
    fi
    return 1
}

test_error_handling() {
    log_info "Testing invalid dimension (2D instead of 384D)..."
    
    local http_code
    http_code=$(curl_cmd -o /tmp/qtest_err.txt -w "%{http_code}" \
        -X POST "$BASE_URL/api/vector/insert" \
        -H "Content-Type: application/json" \
        -d '{"id": "invalid", "vector": [0.1, 0.2], "metadata": {}}' 2>&1)
    
    log_debug "HTTP Code: $http_code"
    
    if [[ "$http_code" == "400" ]]; then
        log_info "Server correctly rejected (HTTP 400)"
        return 0
    fi
    return 1
}

test_bulk_insert() {
    local count=10 success=0
    local start=$(date +%s%N)
    
    log_info "Bulk inserting $count vectors..."
    
    for i in $(seq 1 $count); do
        progress $i $count "Vector $i/$count"
        
        local vector
        vector=$(generate_vector 384)
        local response
        response=$(curl_cmd -X POST "$BASE_URL/api/vector/insert" \
            -H "Content-Type: application/json" \
            -d "{\"id\": \"bulk_$(date +%s)_$i\", \"vector\": $vector, \"metadata\": {\"i\": $i}}" 2>&1)
        
        echo "$response" | jq -e '.success == true' &>/dev/null && ((success++))
    done
    echo ""
    
    local end=$(date +%s%N)
    local duration=$(elapsed_ms $((end - start)))
    local rate=$((count * 1000 / (duration + 1)))
    
    log_info "Result: $success/$count in ${duration}ms (${rate}/sec)"
    [[ $success -ge 8 ]]
}

test_latency() {
    local iterations=5 total_ms=0
    log_info "Measuring latency ($iterations searches)..."
    
    local vector
    vector=$(generate_vector 384)
    
    for i in $(seq 1 $iterations); do
        progress $i $iterations "Search $i/$iterations"
        
        local start=$(date +%s%N)
        curl_cmd -X POST "$BASE_URL/api/vector/search" \
            -H "Content-Type: application/json" \
            -d "{\"vector\": $vector, \"k\": 10}" &>/dev/null
        local end=$(date +%s%N)
        
        total_ms=$((total_ms + $(elapsed_ms $((end - start)))))
    done
    echo ""
    
    local avg=$((total_ms / iterations))
    log_info "Average latency: ${avg}ms"
    [[ $avg -lt 1000 ]]
}

#═══════════════════════════════════════════════════════════════════════════════
# Main
#═══════════════════════════════════════════════════════════════════════════════
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║              QuartzDB Quick Test Suite v2.0                       ║"
    echo "║          Production-Grade • Metrics • Progress                    ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${BOLD}Configuration${NC}"
    echo -e "  Server:   $BASE_URL"
    echo -e "  Timeout:  ${TIMEOUT}s"
    echo -e "  API Key:  ${API_KEY:+${API_KEY:0:8}...}${API_KEY:-none}"
    echo -e "  Verbose:  $VERBOSE"
    echo ""
    
    log_info "Checking server connectivity..."
    if ! curl_cmd "$BASE_URL/health" &>/dev/null; then
        log_fail "Server not reachable at $BASE_URL"
        echo -e "  ${DIM}Start server: cd quartz-faas && wrangler dev${NC}"
        exit 1
    fi
    log_pass "Server is online"
    
    run_test "Health Check" "Verify /health endpoint" test_health
    run_test "Insert Vector" "Insert 384-dim vector with metadata" test_insert
    run_test "Search Vectors" "k-NN search with k=5" test_search
    run_test "Delete Vector" "Soft-delete vector by ID" test_delete
    run_test "Statistics" "Retrieve index statistics" test_stats
    run_test "Error Handling" "Reject invalid dimensions" test_error_handling
    run_test "Bulk Insert" "Insert 10 vectors with progress" test_bulk_insert
    run_test "Latency Benchmark" "Measure search latency (5x)" test_latency
    
    local total_end=$(date +%s%N)
    local total_duration=$(elapsed_ms $((total_end - TOTAL_START)))
    
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║                        Test Summary                               ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    printf "  %-25s %-8s %s\n" "TEST" "STATUS" "TIME"
    printf "  %-25s %-8s %s\n" "─────────────────────────" "──────" "────────"
    for result in "${TEST_RESULTS[@]}"; do
        IFS='|' read -r status name time <<< "$result"
        local color=$([[ "$status" == "PASS" ]] && echo "$GREEN" || echo "$RED")
        printf "  %-25s ${color}%-8s${NC} %sms\n" "$name" "$status" "$time"
    done
    echo ""
    echo -e "  ${BOLD}Total:${NC}    $TOTAL tests"
    echo -e "  ${GREEN}Passed:${NC}   $PASSED"
    echo -e "  ${RED}Failed:${NC}   $FAILED"
    echo -e "  ${BOLD}Duration:${NC} ${total_duration}ms"
    echo ""
    
    if [[ $FAILED -eq 0 ]]; then
        echo -e "  ${GREEN}${BOLD}✓ All tests passed!${NC}"
        exit 0
    else
        echo -e "  ${RED}${BOLD}✗ $FAILED test(s) failed${NC}"
        exit 1
    fi
}

main "$@"
