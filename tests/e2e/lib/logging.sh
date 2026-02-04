#!/bin/bash
set -euo pipefail

LOG_LEVEL="${LOG_LEVEL:-INFO}"
E2E_LOG_DIR="${E2E_LOG_DIR:-/tmp/ftui_e2e_logs}"
E2E_RESULTS_DIR="${E2E_RESULTS_DIR:-/tmp/ftui_e2e_results}"
LOG_FILE="${LOG_FILE:-$E2E_LOG_DIR/e2e.log}"
E2E_JSONL_FILE="${E2E_JSONL_FILE:-$E2E_LOG_DIR/e2e.jsonl}"
E2E_JSONL_DISABLE="${E2E_JSONL_DISABLE:-0}"
E2E_DETERMINISTIC="${E2E_DETERMINISTIC:-0}"

e2e_is_deterministic() {
    [[ "${E2E_DETERMINISTIC:-0}" == "1" ]]
}

e2e_timestamp() {
    if e2e_is_deterministic; then
        local seq="${E2E_TS_COUNTER:-0}"
        seq=$((seq + 1))
        export E2E_TS_COUNTER="$seq"
        printf 'T%06d' "$seq"
        return 0
    fi
    date -Iseconds
}

e2e_run_id() {
    if [[ -n "${E2E_RUN_ID:-}" ]]; then
        printf '%s' "$E2E_RUN_ID"
        return 0
    fi
    if e2e_is_deterministic; then
        local seed="${E2E_SEED:-0}"
        local seq="${E2E_RUN_SEQ:-0}"
        seq=$((seq + 1))
        export E2E_RUN_SEQ="$seq"
        printf 'det_%s_%s' "$seed" "$seq"
        return 0
    fi
    printf 'run_%s_%s' "$(date +%Y%m%d_%H%M%S)" "$$"
}

e2e_run_start_ms() {
    if e2e_is_deterministic; then
        printf '0'
        return 0
    fi
    date +%s%3N
}

e2e_now_ms() {
    if e2e_is_deterministic; then
        local seq="${E2E_MS_COUNTER:-0}"
        seq=$((seq + 100))
        export E2E_MS_COUNTER="$seq"
        printf '%s' "$seq"
        return 0
    fi
    date +%s%3N
}

e2e_log_stamp() {
    if e2e_is_deterministic; then
        local seed="${E2E_SEED:-0}"
        printf 'det_%s' "$seed"
        return 0
    fi
    date +%Y%m%d_%H%M%S
}

e2e_hash_key() {
    local mode="$1"
    local cols="$2"
    local rows="$3"
    local seed="${4:-${E2E_SEED:-0}}"
    printf '%s-%sx%s-seed%s' "$mode" "$cols" "$rows" "$seed"
}

json_escape() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

jsonl_emit() {
    local json="$1"
    if [[ "$E2E_JSONL_DISABLE" == "1" ]]; then
        return 0
    fi
    mkdir -p "$(dirname "$E2E_JSONL_FILE")"
    echo "$json" >> "$E2E_JSONL_FILE"
}

jsonl_init() {
    if [[ "${E2E_JSONL_INIT:-}" == "1" ]]; then
        return 0
    fi
    export E2E_JSONL_INIT=1
    e2e_seed >/dev/null
    export E2E_RUN_ID="${E2E_RUN_ID:-$(e2e_run_id)}"
    export E2E_RUN_START_MS="${E2E_RUN_START_MS:-$(e2e_run_start_ms)}"
    jsonl_env
    jsonl_run_start "${E2E_RUN_CMD:-}"
}

jsonl_env() {
    local ts host rustc cargo git_commit git_dirty
    ts="$(e2e_timestamp)"
    host="$(hostname 2>/dev/null || echo unknown)"
    rustc="$(rustc --version 2>/dev/null || echo unknown)"
    cargo="$(cargo --version 2>/dev/null || echo unknown)"
    git_commit="$(git rev-parse HEAD 2>/dev/null || echo "")"
    if git diff --quiet --ignore-submodules -- 2>/dev/null; then
        git_dirty="false"
    else
        git_dirty="true"
    fi

    local seed_json="null"
    local deterministic_json="false"
    if e2e_is_deterministic; then deterministic_json="true"; fi
    if [[ -n "${E2E_SEED:-}" ]]; then seed_json="${E2E_SEED}"; fi

    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "env" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg host "$host" \
            --arg rustc "$rustc" \
            --arg cargo "$cargo" \
            --arg git_commit "$git_commit" \
            --argjson git_dirty "$git_dirty" \
            --argjson seed "$seed_json" \
            --argjson deterministic "$deterministic_json" \
            --arg term "${TERM:-}" \
            --arg colorterm "${COLORTERM:-}" \
            --arg no_color "${NO_COLOR:-}" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,host:$host,rustc:$rustc,cargo:$cargo,git_commit:$git_commit,git_dirty:$git_dirty,seed:$seed,deterministic:$deterministic,term:$term,colorterm:$colorterm,no_color:$no_color}')"
    else
        jsonl_emit "{\"type\":\"env\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"host\":\"$(json_escape "$host")\",\"rustc\":\"$(json_escape "$rustc")\",\"cargo\":\"$(json_escape "$cargo")\",\"git_commit\":\"$(json_escape "$git_commit")\",\"git_dirty\":${git_dirty},\"seed\":${seed_json},\"deterministic\":${deterministic_json},\"term\":\"$(json_escape "${TERM:-}")\",\"colorterm\":\"$(json_escape "${COLORTERM:-}")\",\"no_color\":\"$(json_escape "${NO_COLOR:-}")\"}"
    fi
}

jsonl_run_start() {
    local cmd="$1"
    local ts
    ts="$(e2e_timestamp)"
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "run_start" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg command "$cmd" \
            --arg log_dir "$E2E_LOG_DIR" \
            --arg results_dir "$E2E_RESULTS_DIR" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,command:$command,log_dir:$log_dir,results_dir:$results_dir}')"
    else
        jsonl_emit "{\"type\":\"run_start\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"command\":\"$(json_escape "$cmd")\",\"log_dir\":\"$(json_escape "$E2E_LOG_DIR")\",\"results_dir\":\"$(json_escape "$E2E_RESULTS_DIR")\"}"
    fi
}

jsonl_run_end() {
    local status="$1"
    local duration_ms="$2"
    local failed_count="$3"
    local ts
    ts="$(e2e_timestamp)"
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "run_end" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg status "$status" \
            --argjson duration_ms "$duration_ms" \
            --argjson failed_count "$failed_count" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,status:$status,duration_ms:$duration_ms,failed_count:$failed_count}')"
    else
        jsonl_emit "{\"type\":\"run_end\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"status\":\"$(json_escape "$status")\",\"duration_ms\":${duration_ms},\"failed_count\":${failed_count}}"
    fi
}

jsonl_set_context() {
    export E2E_CONTEXT_MODE="${1:-${E2E_CONTEXT_MODE:-}}"
    export E2E_CONTEXT_COLS="${2:-${E2E_CONTEXT_COLS:-}}"
    export E2E_CONTEXT_ROWS="${3:-${E2E_CONTEXT_ROWS:-}}"
    export E2E_CONTEXT_SEED="${4:-${E2E_CONTEXT_SEED:-}}"
}

e2e_seed() {
    local seed="${E2E_SEED:-0}"
    export E2E_SEED="$seed"
    if e2e_is_deterministic; then
        if [[ -z "${FTUI_SEED:-}" ]]; then
            export FTUI_SEED="$seed"
        fi
        if [[ -z "${FTUI_HARNESS_SEED:-}" ]]; then
            export FTUI_HARNESS_SEED="$seed"
        fi
    fi
    if [[ -z "${E2E_CONTEXT_SEED:-}" ]]; then
        export E2E_CONTEXT_SEED="$seed"
    fi
    printf '%s' "$seed"
}

jsonl_step_start() {
    local step="$1"
    local ts
    ts="$(e2e_timestamp)"
    local mode="${E2E_CONTEXT_MODE:-}"
    local cols="${E2E_CONTEXT_COLS:-}"
    local rows="${E2E_CONTEXT_ROWS:-}"
    local seed="${E2E_CONTEXT_SEED:-}"
    local hash_key=""
    if [[ -n "$mode" && -n "$cols" && -n "$rows" ]]; then
        hash_key="$(e2e_hash_key "$mode" "$cols" "$rows" "$seed")"
    fi
    local cols_json="null"
    local rows_json="null"
    local seed_json="null"
    if [[ -n "$cols" ]]; then cols_json="$cols"; fi
    if [[ -n "$rows" ]]; then rows_json="$rows"; fi
    if [[ -n "$seed" ]]; then seed_json="$seed"; fi
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "step_start" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg step "$step" \
            --arg mode "$mode" \
            --arg hash_key "$hash_key" \
            --argjson cols "$cols_json" \
            --argjson rows "$rows_json" \
            --argjson seed "$seed_json" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,step:$step,mode:$mode,hash_key:$hash_key,cols:$cols,rows:$rows,seed:$seed}')"
    else
        jsonl_emit "{\"type\":\"step_start\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"step\":\"$(json_escape "$step")\",\"mode\":\"$(json_escape "$mode")\",\"hash_key\":\"$(json_escape "$hash_key")\",\"cols\":${cols_json},\"rows\":${rows_json},\"seed\":${seed_json}}"
    fi
}

jsonl_step_end() {
    local step="$1"
    local status="$2"
    local duration_ms="$3"
    local ts
    ts="$(e2e_timestamp)"
    local mode="${E2E_CONTEXT_MODE:-}"
    local cols="${E2E_CONTEXT_COLS:-}"
    local rows="${E2E_CONTEXT_ROWS:-}"
    local seed="${E2E_CONTEXT_SEED:-}"
    local hash_key=""
    if [[ -n "$mode" && -n "$cols" && -n "$rows" ]]; then
        hash_key="$(e2e_hash_key "$mode" "$cols" "$rows" "$seed")"
    fi
    local cols_json="null"
    local rows_json="null"
    local seed_json="null"
    if [[ -n "$cols" ]]; then cols_json="$cols"; fi
    if [[ -n "$rows" ]]; then rows_json="$rows"; fi
    if [[ -n "$seed" ]]; then seed_json="$seed"; fi
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "step_end" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg step "$step" \
            --arg status "$status" \
            --argjson duration_ms "$duration_ms" \
            --arg mode "$mode" \
            --arg hash_key "$hash_key" \
            --argjson cols "$cols_json" \
            --argjson rows "$rows_json" \
            --argjson seed "$seed_json" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,step:$step,status:$status,duration_ms:$duration_ms,mode:$mode,hash_key:$hash_key,cols:$cols,rows:$rows,seed:$seed}')"
    else
        jsonl_emit "{\"type\":\"step_end\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"step\":\"$(json_escape "$step")\",\"status\":\"$(json_escape "$status")\",\"duration_ms\":${duration_ms},\"mode\":\"$(json_escape "$mode")\",\"hash_key\":\"$(json_escape "$hash_key")\",\"cols\":${cols_json},\"rows\":${rows_json},\"seed\":${seed_json}}"
    fi
}

jsonl_pty_capture() {
    local output_file="$1"
    local cols="$2"
    local rows="$3"
    local exit_code="$4"
    local canonical_file="${5:-}"
    jsonl_init
    local ts output_sha output_bytes canonical_sha canonical_bytes
    ts="$(e2e_timestamp)"
    output_sha="$(sha256_file "$output_file")"
    output_bytes=$(wc -c < "$output_file" 2>/dev/null | tr -d ' ')
    canonical_sha=""
    canonical_bytes=0
    if [[ -n "$canonical_file" && -f "$canonical_file" ]]; then
        canonical_sha="$(sha256_file "$canonical_file")"
        canonical_bytes=$(wc -c < "$canonical_file" 2>/dev/null | tr -d ' ')
    fi
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "pty_capture" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg output_file "$output_file" \
            --arg canonical_file "$canonical_file" \
            --arg output_sha256 "$output_sha" \
            --arg canonical_sha256 "$canonical_sha" \
            --argjson output_bytes "${output_bytes:-0}" \
            --argjson canonical_bytes "${canonical_bytes:-0}" \
            --argjson cols "$cols" \
            --argjson rows "$rows" \
            --argjson exit_code "$exit_code" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,output_file:$output_file,canonical_file:$canonical_file,output_sha256:$output_sha256,canonical_sha256:$canonical_sha256,output_bytes:$output_bytes,canonical_bytes:$canonical_bytes,cols:$cols,rows:$rows,exit_code:$exit_code}')"
    else
        jsonl_emit "{\"type\":\"pty_capture\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"output_file\":\"$(json_escape "$output_file")\",\"canonical_file\":\"$(json_escape "$canonical_file")\",\"output_sha256\":\"$(json_escape "$output_sha")\",\"canonical_sha256\":\"$(json_escape "$canonical_sha")\",\"output_bytes\":${output_bytes:-0},\"canonical_bytes\":${canonical_bytes:-0},\"cols\":${cols},\"rows\":${rows},\"exit_code\":${exit_code}}"
    fi
}

jsonl_assert() {
    local name="$1"
    local status="$2"
    local details="${3:-}"
    local ts
    ts="$(e2e_timestamp)"
    if command -v jq >/dev/null 2>&1; then
        jsonl_emit "$(jq -nc \
            --arg type "assert" \
            --arg timestamp "$ts" \
            --arg run_id "$E2E_RUN_ID" \
            --arg assertion "$name" \
            --arg status "$status" \
            --arg details "$details" \
            '{type:$type,timestamp:$timestamp,run_id:$run_id,assertion:$assertion,status:$status,details:$details}')"
    else
        jsonl_emit "{\"type\":\"assert\",\"timestamp\":\"$(json_escape "$ts")\",\"run_id\":\"$(json_escape "$E2E_RUN_ID")\",\"assertion\":\"$(json_escape "$name")\",\"status\":\"$(json_escape "$status")\",\"details\":\"$(json_escape "$details")\"}"
    fi
}

sha256_file() {
    local file="$1"
    if command -v sha256sum >/dev/null 2>&1 && [[ -f "$file" ]]; then
        sha256sum "$file" | awk '{print $1}'
        return 0
    fi
    return 1
}

verify_sha256() {
    local file="$1"
    local expected="$2"
    local label="${3:-sha256_match}"
    local actual=""
    actual="$(sha256_file "$file" || true)"
    if [[ -z "$actual" ]]; then
        jsonl_assert "$label" "skipped" "sha256sum unavailable or file missing"
        return 2
    fi
    if [[ "$actual" == "$expected" ]]; then
        jsonl_assert "$label" "passed" "sha256 match"
        return 0
    fi
    jsonl_assert "$label" "failed" "expected ${expected}, got ${actual}"
    return 1
}

log() {
    local level="$1"
    shift
    local ts
    ts="$(date +"%Y-%m-%d %H:%M:%S.%3N")"
    echo "[$ts] [$level] $*" | tee -a "$LOG_FILE"
}

log_debug() {
    if [[ "$LOG_LEVEL" == "DEBUG" ]]; then
        log "DEBUG" "$@"
    fi
}

log_info() {
    log "INFO" "$@"
}

log_warn() {
    log "WARN" "$@"
}

log_error() {
    log "ERROR" "$@"
}

log_test_start() {
    local name="$1"
    jsonl_init
    jsonl_step_start "$name"
    log_info "========================================"
    log_info "STARTING TEST: $name"
    log_info "========================================"
}

log_test_pass() {
    local name="$1"
    log_info "PASS: $name"
}

log_test_fail() {
    local name="$1"
    local reason="$2"
    log_error "FAIL: $name"
    log_error "  Reason: $reason"
    log_error "  Log file: $LOG_FILE"
}

log_test_skip() {
    local name="$1"
    local reason="$2"
    log_warn "SKIP: $name"
    log_warn "  Reason: $reason"
}

record_result() {
    local name="$1"
    local status="$2"
    local duration_ms="$3"
    local log_file="$4"
    local error_msg="${5:-}"
    jsonl_init

    mkdir -p "$E2E_RESULTS_DIR"

    local result_file
    result_file="$E2E_RESULTS_DIR/${name}_$(date +%s%N)_$$.json"

    if command -v jq >/dev/null 2>&1; then
        if [[ -n "$error_msg" ]]; then
            jq -n \
                --arg name "$name" \
                --arg status "$status" \
                --argjson duration_ms "$duration_ms" \
                --arg log_file "$log_file" \
                --arg error "$error_msg" \
                '{name:$name,status:$status,duration_ms:$duration_ms,log_file:$log_file,error:$error}' \
                > "$result_file"
        else
            jq -n \
                --arg name "$name" \
                --arg status "$status" \
                --argjson duration_ms "$duration_ms" \
                --arg log_file "$log_file" \
                '{name:$name,status:$status,duration_ms:$duration_ms,log_file:$log_file}' \
                > "$result_file"
        fi
    else
        local safe_error
        safe_error="$(printf '%s' "$error_msg" | sed 's/"/\\"/g')"
        if [[ -n "$safe_error" ]]; then
            printf '{"name":"%s","status":"%s","duration_ms":%s,"log_file":"%s","error":"%s"}\n' \
                "$name" "$status" "$duration_ms" "$log_file" "$safe_error" \
                > "$result_file"
        else
            printf '{"name":"%s","status":"%s","duration_ms":%s,"log_file":"%s"}\n' \
                "$name" "$status" "$duration_ms" "$log_file" \
                > "$result_file"
        fi
    fi
    jsonl_step_end "$name" "$status" "$duration_ms"
}

finalize_summary() {
    local summary_file="$1"
    local end_ms
    end_ms="$(e2e_now_ms)"
    local start_ms="${E2E_RUN_START_MS:-$end_ms}"
    local duration_ms=$((end_ms - start_ms))

    if command -v jq >/dev/null 2>&1; then
        jq -s \
            --arg timestamp "$(e2e_timestamp)" \
            --argjson duration_ms "$duration_ms" \
            '{
                timestamp: $timestamp,
                total: length,
                passed: (map(select(.status=="passed")) | length),
                failed: (map(select(.status=="failed")) | length),
                skipped: (map(select(.status=="skipped")) | length),
                duration_ms: $duration_ms,
                tests: .
            }' \
            "$E2E_RESULTS_DIR"/*.json > "$summary_file"
    else
        local total
        total=$(ls -1 "$E2E_RESULTS_DIR"/*.json 2>/dev/null | wc -l | tr -d ' ')
        cat > "$summary_file" <<EOF_SUM
{"timestamp":"$(e2e_timestamp)","total":${total},"passed":0,"failed":0,"skipped":0,"duration_ms":${duration_ms},"tests":[]}
EOF_SUM
    fi
    local failed_count=0
    if command -v jq >/dev/null 2>&1; then
        failed_count=$(jq '.failed // 0' "$summary_file" 2>/dev/null || echo 0)
    fi
    if [[ "$failed_count" -gt 0 ]]; then
        jsonl_run_end "failed" "$duration_ms" "$failed_count"
    else
        jsonl_run_end "complete" "$duration_ms" "$failed_count"
    fi
}
