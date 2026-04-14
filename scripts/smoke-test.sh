#!/bin/bash
set -e
BASE=${1:-"http://localhost:3000"}
FRONTEND=${2:-"http://localhost:5173"}
PASS=0; FAIL=0

check() {
  local name=$1 url=$2 expected=$3
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$url")
  if [ "$STATUS" = "$expected" ]; then
    echo "[PASS] $name ($STATUS)"
    PASS=$((PASS+1))
  else
    echo "[FAIL] $name (expected $expected, got $STATUS)"
    FAIL=$((FAIL+1))
  fi
}

echo "=== Smoke Test ==="
echo "Backend: $BASE"
echo "Frontend: $FRONTEND"
echo ""

# Backend health
check "API health" "$BASE/api/health" "200"

# Auth endpoints (POST-only, expect 400/422 with empty body)
check_post() {
  local name=$1 url=$2 expected=$3
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -d '{}' "$url")
  if [ "$STATUS" = "$expected" ]; then
    echo "[PASS] $name ($STATUS)"
    PASS=$((PASS+1))
  else
    echo "[FAIL] $name (expected $expected, got $STATUS)"
    FAIL=$((FAIL+1))
  fi
}

check_post "Login endpoint reachable" "$BASE/api/auth/member/login" "422"
check_post "Register endpoint reachable" "$BASE/api/subjects/register" "422"

# Protected endpoints (should return 401 without token)
check "Me requires auth" "$BASE/api/subjects/me" "401"
check "Sessions requires auth" "$BASE/api/sessions" "401"
check "Credentials requires auth" "$BASE/api/credentials/status" "401"

# Frontend
check "Frontend homepage" "$FRONTEND" "200"

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
echo "=== All smoke checks passed ==="
