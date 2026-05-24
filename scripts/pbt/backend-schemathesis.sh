#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

host="${BACKEND_PBT_HOST:-127.0.0.1}"
port="${BACKEND_PBT_PORT:-3011}"
base_url="${BACKEND_PBT_BASE_URL:-http://${host}:${port}}"
snapshot="${BACKEND_PBT_SNAPSHOT:-${repo_root}/tests/fixtures/backend-pbt/snapshot.json}"
fasta="${BACKEND_PBT_FASTA:-${repo_root}/tests/fixtures/backend-pbt/reference.fa}"
schemathesis_version="${SCHEMATHESIS_VERSION:-4.18.0}"
report_dir="${repo_root}/target/schemathesis"
api_log="${report_dir}/api.log"
api_pid=""

cleanup() {
	if [[ -n "${api_pid}" ]] && kill -0 "${api_pid}" 2>/dev/null; then
		kill "${api_pid}" 2>/dev/null || true
		wait "${api_pid}" 2>/dev/null || true
	fi
}
trap cleanup EXIT INT TERM

mkdir -p "${report_dir}"

if [[ "${BACKEND_PBT_START_SERVER:-1}" != "0" ]]; then
	cargo run --locked -p api -- \
		--bind "${host}:${port}" \
		--snapshot "${snapshot}" \
		--fasta "${fasta}" >"${api_log}" 2>&1 &
	api_pid="$!"

	for _ in {1..80}; do
		if curl -fsS "${base_url}/health" >/dev/null 2>&1; then
			break
		fi
		if ! kill -0 "${api_pid}" 2>/dev/null; then
			wait "${api_pid}" || true
			sed -n '1,160p' "${api_log}" >&2
			exit 1
		fi
		sleep 0.25
	done

	if ! curl -fsS "${base_url}/health" >/dev/null 2>&1; then
		sed -n '1,160p' "${api_log}" >&2
		echo "API did not become healthy at ${base_url}" >&2
		exit 1
	fi
fi

cd "${repo_root}"
uvx --from "schemathesis==${schemathesis_version}" schemathesis \
	--config-file "${repo_root}/schemathesis.toml" \
	run "${base_url}/openapi.json" "$@"
