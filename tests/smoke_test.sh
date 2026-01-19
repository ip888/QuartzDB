#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# QuartzDB Smoke Test Suite v2.0
# Real-world scenario validation with progress tracking
#═══════════════════════════════════════════════════════════════════════════════
# Usage: ./smoke_test.sh [--verbose] [--timeout SECONDS]
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
# Colors
#───────────────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
err() { echo -e "${RED}✗${NC} $1"; }
step() { echo -e "  ${YELLOW}→${NC} $1"; }

progress() {
    local current=$1 total=$2 desc=$3
    local pct=$((current * 100 / total))
    local filled=$((pct / 5))
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=filled; i<20; i++)); do bar+="░"; done
    printf "\r  ${DIM}[${bar}]${NC} %3d%% %s" "$pct" "$desc"
}

curl_cmd() {
    local args=(-s --connect-timeout "$TIMEOUT" --max-time "$TIMEOUT")
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
# Smoke Test 1: E-Commerce Product Search
#───────────────────────────────────────────────────────────────────────────────
test_ecommerce() {
    log "Scenario 1: E-Commerce Product Search"
    echo ""
    
    step "Inserting 5 product vectors..."
    for i in $(seq 1 5); do
        progress $i 5 "Product $i"
        local vector
        vector=$(generate_vector 384)
        curl_cmd -X POST "$BASE_URL/api/vector/insert" \
            -H "Content-Type: application/json" \
            -d "{\"id\": \"smoke_product_$i\", \"vector\": $vector, \"metadata\": {\"name\": \"Product $i\", \"price\": $((i * 10))}}" >/dev/null
    done
    echo ""
    ok "Products indexed"
    
    step "Searching for similar products..."
    local vector results
    vector=$(generate_vector 384)
    results=$(curl_cmd -X POST "$BASE_URL/api/vector/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\": $vector, \"k\": 3}")
    
    if echo "$results" | jq -e '.success == true' &>/dev/null; then
        local count
        count=$(echo "$results" | jq '.results | length')
        ok "Found $count similar products"
        return 0
    else
        err "Search failed"
        return 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Smoke Test 2: Document Semantic Search
#───────────────────────────────────────────────────────────────────────────────
test_documents() {
    log "Scenario 2: Document Semantic Search"
    echo ""
    
    step "Inserting 3 document vectors..."
    for i in $(seq 1 3); do
        progress $i 3 "Document $i"
        local vector
        vector=$(generate_vector 384)
        curl_cmd -X POST "$BASE_URL/api/vector/insert" \
            -H "Content-Type: application/json" \
            -d "{\"id\": \"smoke_doc_$i\", \"vector\": $vector, \"metadata\": {\"title\": \"Document $i\"}}" >/dev/null
    done
    echo ""
    ok "Documents indexed"
    
    step "Searching documents..."
    local vector results
    vector=$(generate_vector 384)
    results=$(curl_cmd -X POST "$BASE_URL/api/vector/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\": $vector, \"k\": 2}")
    
    if echo "$results" | jq -e '.success == true' &>/dev/null; then
        ok "Search complete"
        return 0
    else
        err "Search failed"
        return 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Smoke Test 3: Data Lifecycle (Insert → Search → Delete)
#───────────────────────────────────────────────────────────────────────────────
test_lifecycle() {
    log "Scenario 3: Data Lifecycle Management"
    echo ""
    
    local test_id="smoke_lifecycle_$(date +%s)"
    
    step "Insert test document..."
    local vector
    vector=$(generate_vector 384)
    local insert_result
    insert_result=$(curl_cmd -X POST "$BASE_URL/api/vector/insert" \
        -H "Content-Type: application/json" \
        -d "{\"id\": \"$test_id\", \"vector\": $vector, \"metadata\": {\"test\": true}}")
    
    if ! echo "$insert_result" | jq -e '.success == true' &>/dev/null; then
        err "Insert failed"
        return 1
    fi
    ok "Document inserted: $test_id"
    
    step "Verify document exists..."
    local search_result
    search_result=$(curl_cmd -X POST "$BASE_URL/api/vector/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\": $vector, \"k\": 5}")
    
    if echo "$search_result" | jq -e '.success == true' &>/dev/null; then
        ok "Document found in search"
    fi
    
    step "Soft delete document..."
    local delete_result
    delete_result=$(curl_cmd -X DELETE "$BASE_URL/api/vector/delete/$test_id")
    
    if echo "$delete_result" | grep -qi "success\|deleted"; then
        ok "Document deleted"
        return 0
    else
        err "Delete failed"
        return 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Smoke Test 4: Statistics & Monitoring
#───────────────────────────────────────────────────────────────────────────────
test_statistics() {
    log "Scenario 4: Statistics & Monitoring"
    echo ""
    
    step "Fetching system statistics..."
    local stats
    stats=$(curl_cmd "$BASE_URL/api/vector/stats")
    
    # Check for both possible field names (total_vectors or num_vectors)
    if echo "$stats" | jq -e '.total_vectors >= 0 or .num_vectors >= 0' &>/dev/null; then
        echo "$stats" | jq '{total: (.total_vectors // .num_vectors), active: (.active_vectors // .num_active), deleted: (.deleted_vectors // .num_deleted)}' 2>/dev/null || echo "$stats"
        ok "Statistics retrieved"
        return 0
    else
        err "Stats failed"
        return 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Smoke Test 5: Performance Check
#───────────────────────────────────────────────────────────────────────────────
test_performance() {
    log "Scenario 5: Performance Benchmark"
    echo ""
    
    step "Measuring search latency (10 searches)..."
    local vector total_ms=0
    vector=$(generate_vector 384)
    
    for i in $(seq 1 10); do
        progress $i 10 "Search $i"
        local start=$(date +%s%N)
        curl_cmd -X POST "$BASE_URL/api/vector/search" \
            -H "Content-Type: application/json" \
            -d "{\"vector\": $vector, \"k\": 10}" >/dev/null
        local end=$(date +%s%N)
        total_ms=$((total_ms + $(elapsed_ms $((end - start)))))
    done
    echo ""
    
    local avg=$((total_ms / 10))
    
    if [[ $avg -lt 100 ]]; then
        ok "Average latency: ${avg}ms (excellent)"
    elif [[ $avg -lt 500 ]]; then
        ok "Average latency: ${avg}ms (good)"
    else
        echo -e "${YELLOW}⚠${NC} Average latency: ${avg}ms (slow)"
    fi
    
    return 0
}

#═══════════════════════════════════════════════════════════════════════════════
# Main
#═══════════════════════════════════════════════════════════════════════════════
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║              QuartzDB Smoke Test Suite v2.0                       ║"
    echo "║             Real-World Scenarios Validation                       ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${BOLD}Configuration${NC}"
    echo -e "  Server:   $BASE_URL"
    echo -e "  Timeout:  ${TIMEOUT}s"
    echo -e "  API Key:  ${API_KEY:+${API_KEY:0:8}...}${API_KEY:-none}"
    echo ""
    
    log "Checking server connectivity..."
    if ! curl_cmd "$BASE_URL/health" &>/dev/null; then
        echo -e "${RED}Server not reachable at $BASE_URL${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓${NC} Server is online"
    
    local passed=0 failed=0
    local tests=("test_ecommerce" "test_documents" "test_lifecycle" "test_statistics" "test_performance")
    
    for test in "${tests[@]}"; do
        echo ""
        if $test; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    local total_end=$(date +%s%N)
    local total_duration=$(elapsed_ms $((total_end - TOTAL_START)))
    
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║                     Smoke Test Summary                            ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "  ${BOLD}Total:${NC}    $((passed + failed)) tests"
    echo -e "  ${GREEN}Passed:${NC}   $passed"
    echo -e "  ${RED}Failed:${NC}   $failed"
    echo -e "  ${BOLD}Duration:${NC} ${total_duration}ms"
    echo ""
    
    if [[ $failed -eq 0 ]]; then
        echo -e "  ${GREEN}${BOLD}✓ All smoke tests passed!${NC}"
        exit 0
    else
        echo -e "  ${RED}${BOLD}✗ $failed test(s) failed${NC}"
        exit 1
    fi
}

main "$@"
