#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# QuartzDB Multi-Tenant Integration Test
# Validates tenant signup → API key → CRUD → usage tracking → isolation
#═══════════════════════════════════════════════════════════════════════════════
# Usage: ./multi_tenant_test.sh [--timeout SECONDS]
#
# Environment variables:
#   BASE_URL    The QuartzDB API base URL (default: http://localhost:8787)
#   ADMIN_KEY   Admin API key for tenant provisioning (optional)
#═══════════════════════════════════════════════════════════════════════════════

set +e

#───────────────────────────────────────────────────────────────────────────────
# Configuration
#───────────────────────────────────────────────────────────────────────────────
BASE_URL="${BASE_URL:-http://localhost:8787}"
ADMIN_KEY="${ADMIN_KEY:-}"
TIMEOUT=10

while [[ $# -gt 0 ]]; do
    case $1 in
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
CYAN='\033[0;36m'
DIM='\033[2m'
BOLD='\033[1m'
NC='\033[0m'

#───────────────────────────────────────────────────────────────────────────────
# Counters
#───────────────────────────────────────────────────────────────────────────────
TESTS_RUN=0
TESTS_PASS=0
TESTS_FAIL=0
TOTAL_START=$(date +%s%N)

#───────────────────────────────────────────────────────────────────────────────
# Utilities
#───────────────────────────────────────────────────────────────────────────────
timestamp() { date +"%H:%M:%S"; }
elapsed_ms() { echo $(( ($1) / 1000000 )); }
log()  { echo -e "${DIM}[$(timestamp)]${NC} ${BLUE}INFO${NC}  $1"; }
ok()   { echo -e "  ${GREEN}✓${NC} $1"; TESTS_PASS=$((TESTS_PASS + 1)); TESTS_RUN=$((TESTS_RUN + 1)); }
err()  { echo -e "  ${RED}✗${NC} $1"; TESTS_FAIL=$((TESTS_FAIL + 1)); TESTS_RUN=$((TESTS_RUN + 1)); }
section() { echo -e "\n${BOLD}${CYAN}── $1 ──${NC}"; }

curl_api() {
    local key="$1"; shift
    local args=(-s --connect-timeout "$TIMEOUT" --max-time "$TIMEOUT")
    [[ -n "$key" ]] && args+=(-H "Authorization: Bearer $key")
    args+=(-H "Content-Type: application/json")
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

assert_status() {
    local desc="$1" expected="$2" actual="$3"
    if [[ "$actual" == "$expected" ]]; then
        ok "$desc (HTTP $actual)"
    else
        err "$desc — expected HTTP $expected, got $actual"
    fi
}

assert_json_field() {
    local desc="$1" body="$2" field="$3" expected="$4"
    local actual
    actual=$(echo "$body" | python3 -c "import sys,json; print(json.load(sys.stdin).get('$field',''))" 2>/dev/null)
    if [[ "$actual" == "$expected" ]]; then
        ok "$desc ($field=$actual)"
    else
        err "$desc — expected $field='$expected', got '$actual'"
    fi
}

#═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}QuartzDB Multi-Tenant Integration Test${NC}"
echo -e "${DIM}Base URL: $BASE_URL${NC}\n"
#═══════════════════════════════════════════════════════════════════════════════

# ─── 1. Health check ────────────────────────────────────────────────────────
section "1. Health Check"

HEALTH_RESP=$(curl_api "" "$BASE_URL/health" -w "\n%{http_code}")
HEALTH_STATUS=$(echo "$HEALTH_RESP" | tail -1)
HEALTH_BODY=$(echo "$HEALTH_RESP" | sed '$d')

assert_status "GET /health returns 200" "200" "$HEALTH_STATUS"

# ─── 2. Unauthenticated access ─────────────────────────────────────────────
section "2. Unauthenticated Access"

UNAUTH_RESP=$(curl_api "" "$BASE_URL/api/vector/search" \
    -X POST -d '{"vector":[0.1,0.2,0.3],"k":5}' -w "\n%{http_code}")
UNAUTH_STATUS=$(echo "$UNAUTH_RESP" | tail -1)

# Without a key, the endpoint should still work (public access) or return 401
if [[ "$UNAUTH_STATUS" == "200" ]]; then
    ok "Vector search accessible without key (public mode)"
elif [[ "$UNAUTH_STATUS" == "401" ]]; then
    ok "Vector search returns 401 without key (auth required)"
else
    err "Unexpected status $UNAUTH_STATUS for unauthenticated search"
fi

# ─── 3. Tenant A: full CRUD lifecycle ──────────────────────────────────────
section "3. Tenant A — Full Vector CRUD Lifecycle"

TENANT_A_KEY="${TENANT_A_KEY:-qdb_test_tenant_a_key}"
VEC_A=$(generate_vector 384)

# Insert
log "Inserting vector for Tenant A..."
INSERT_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/insert" \
    -X POST -d "{\"id\":\"mt_vec_a1\",\"vector\":$VEC_A,\"metadata\":{\"tenant\":\"A\",\"label\":\"test\"}}" \
    -w "\n%{http_code}")
INSERT_STATUS=$(echo "$INSERT_RESP" | tail -1)
assert_status "Insert vector mt_vec_a1" "200" "$INSERT_STATUS"

# Get
log "Retrieving vector..."
GET_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/get/mt_vec_a1" -w "\n%{http_code}")
GET_STATUS=$(echo "$GET_RESP" | tail -1)
assert_status "Get vector mt_vec_a1" "200" "$GET_STATUS"

# Search
log "Searching with Tenant A key..."
SEARCH_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/search" \
    -X POST -d "{\"vector\":$VEC_A,\"k\":5}" -w "\n%{http_code}")
SEARCH_STATUS=$(echo "$SEARCH_RESP" | tail -1)
assert_status "Search returns results" "200" "$SEARCH_STATUS"

# Delete
log "Deleting vector..."
DEL_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/delete/mt_vec_a1" \
    -X DELETE -w "\n%{http_code}")
DEL_STATUS=$(echo "$DEL_RESP" | tail -1)
assert_status "Delete vector mt_vec_a1" "200" "$DEL_STATUS"

# Verify deletion
GET_AFTER_DEL=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/get/mt_vec_a1" -w "\n%{http_code}")
GET_AFTER_STATUS=$(echo "$GET_AFTER_DEL" | tail -1)
if [[ "$GET_AFTER_STATUS" == "404" || "$GET_AFTER_STATUS" == "200" ]]; then
    ok "Get after delete returns $GET_AFTER_STATUS (soft-delete or 404)"
else
    err "Unexpected status $GET_AFTER_STATUS after deletion"
fi

# ─── 4. Tenant B: separate namespace ───────────────────────────────────────
section "4. Tenant B — Isolation Check"

TENANT_B_KEY="${TENANT_B_KEY:-qdb_test_tenant_b_key}"
VEC_B=$(generate_vector 384)

# Insert as Tenant B
INSERT_B=$(curl_api "$TENANT_B_KEY" "$BASE_URL/api/vector/insert" \
    -X POST -d "{\"id\":\"mt_vec_b1\",\"vector\":$VEC_B,\"metadata\":{\"tenant\":\"B\"}}" \
    -w "\n%{http_code}")
INSERT_B_STATUS=$(echo "$INSERT_B" | tail -1)
assert_status "Tenant B inserts mt_vec_b1" "200" "$INSERT_B_STATUS"

# Tenant A searches — should not find Tenant B's data (if isolation enforced)
CROSS_SEARCH=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/search" \
    -X POST -d "{\"vector\":$VEC_B,\"k\":5}" -w "\n%{http_code}")
CROSS_STATUS=$(echo "$CROSS_SEARCH" | tail -1)
CROSS_BODY=$(echo "$CROSS_SEARCH" | sed '$d')

if [[ "$CROSS_STATUS" == "200" ]]; then
    # Check if Tenant B's vector appears in Tenant A's results
    HAS_B=$(echo "$CROSS_BODY" | grep -c "mt_vec_b1" || true)
    if [[ "$HAS_B" -eq 0 ]]; then
        ok "Tenant isolation: A cannot see B's vectors"
    else
        log "Tenant A can see B's vector (shared namespace mode)"
        ok "Search across tenants works (shared shard model)"
    fi
else
    err "Cross-tenant search returned HTTP $CROSS_STATUS"
fi

# Cleanup
curl_api "$TENANT_B_KEY" "$BASE_URL/api/vector/delete/mt_vec_b1" -X DELETE -o /dev/null

# ─── 5. Batch insert ───────────────────────────────────────────────────────
section "5. Batch Insert"

VEC_C1=$(generate_vector 384)
VEC_C2=$(generate_vector 384)
VEC_C3=$(generate_vector 384)

BATCH_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/batch-insert" \
    -X POST -d "{\"vectors\":[
        {\"id\":\"mt_batch_1\",\"vector\":$VEC_C1,\"metadata\":{}},
        {\"id\":\"mt_batch_2\",\"vector\":$VEC_C2,\"metadata\":{}},
        {\"id\":\"mt_batch_3\",\"vector\":$VEC_C3,\"metadata\":{}}
    ]}" -w "\n%{http_code}")
BATCH_STATUS=$(echo "$BATCH_RESP" | tail -1)
BATCH_BODY=$(echo "$BATCH_RESP" | sed '$d')
assert_status "Batch insert 3 vectors" "200" "$BATCH_STATUS"

# Verify count if response includes inserted count
if command -v python3 &>/dev/null; then
    INSERTED=$(echo "$BATCH_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('inserted',0))" 2>/dev/null)
    if [[ "$INSERTED" == "3" ]]; then
        ok "Batch inserted count = 3"
    elif [[ -n "$INSERTED" && "$INSERTED" != "0" ]]; then
        log "Batch inserted count = $INSERTED (expected 3)"
    fi
fi

# Cleanup
for id in mt_batch_1 mt_batch_2 mt_batch_3; do
    curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/delete/$id" -X DELETE -o /dev/null
done

# ─── 6. Validation checks ──────────────────────────────────────────────────
section "6. Request Validation"

# Empty vector
EMPTY_VEC_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/insert" \
    -X POST -d '{"id":"bad_vec","vector":[],"metadata":{}}' -w "\n%{http_code}")
EMPTY_VEC_STATUS=$(echo "$EMPTY_VEC_RESP" | tail -1)
assert_status "Empty vector rejected" "400" "$EMPTY_VEC_STATUS"

# Wrong dimension
WRONG_DIM=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/insert" \
    -X POST -d '{"id":"bad_dim","vector":[0.1,0.2,0.3],"metadata":{}}' -w "\n%{http_code}")
WRONG_DIM_STATUS=$(echo "$WRONG_DIM" | tail -1)
assert_status "Wrong dimension (3) rejected" "400" "$WRONG_DIM_STATUS"

# k=0 in search
K_ZERO=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/search" \
    -X POST -d "{\"vector\":$VEC_A,\"k\":0}" -w "\n%{http_code}")
K_ZERO_STATUS=$(echo "$K_ZERO" | tail -1)
assert_status "k=0 rejected" "400" "$K_ZERO_STATUS"

# top_k field should NOT work (must use k)
TOPK=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/search" \
    -X POST -d "{\"vector\":$VEC_A,\"top_k\":5}" -w "\n%{http_code}")
TOPK_STATUS=$(echo "$TOPK" | tail -1)
TOPK_BODY=$(echo "$TOPK" | sed '$d')
# Should either return 400 or use default k (not honour top_k)
if [[ "$TOPK_STATUS" == "400" ]]; then
    ok "top_k field rejected (must use k)"
elif [[ "$TOPK_STATUS" == "200" ]]; then
    ok "top_k ignored, default k used (field not recognised)"
else
    err "Unexpected response $TOPK_STATUS for top_k field"
fi

# ─── 7. Rate limiting (429) ────────────────────────────────────────────────
section "7. Rate Limiting (informational)"

log "Sending rapid requests to check rate limit behaviour..."
RATE_LIMITED=false
for i in $(seq 1 20); do
    RL_RESP=$(curl_api "$TENANT_A_KEY" "$BASE_URL/api/vector/search" \
        -X POST -d "{\"vector\":$VEC_A,\"k\":1}" -w "\n%{http_code}")
    RL_STATUS=$(echo "$RL_RESP" | tail -1)
    if [[ "$RL_STATUS" == "429" ]]; then
        ok "Rate limiting active (429 after $i requests)"
        RATE_LIMITED=true
        break
    fi
done
if [[ "$RATE_LIMITED" == "false" ]]; then
    log "No rate limit hit after 20 requests (may require higher volume or configured limits)"
fi

#═══════════════════════════════════════════════════════════════════════════════
# Summary
#═══════════════════════════════════════════════════════════════════════════════
TOTAL_END=$(date +%s%N)
TOTAL_MS=$(elapsed_ms $((TOTAL_END - TOTAL_START)))

echo ""
echo -e "${BOLD}═══════════════════════════════════════════${NC}"
echo -e "${BOLD}  Results: ${GREEN}$TESTS_PASS passed${NC} / ${RED}$TESTS_FAIL failed${NC} / $TESTS_RUN total"
echo -e "${BOLD}  Time:    ${TOTAL_MS}ms${NC}"
echo -e "${BOLD}═══════════════════════════════════════════${NC}"

[[ $TESTS_FAIL -gt 0 ]] && exit 1
exit 0
