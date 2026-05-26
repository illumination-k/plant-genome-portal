#!/usr/bin/env bash
# Shared helpers for atacseq pipeline tests.
#
# Sets NEXTFLOW to the binary path, downloading a pinned release into a local
# cache the first time it is needed.

# shellcheck shell=bash

NEXTFLOW_VERSION="${NEXTFLOW_VERSION:-24.10.4}"

ensure_nextflow() {
	if [ -n "${NEXTFLOW:-}" ] && [ -x "${NEXTFLOW}" ]; then
		return
	fi
	if command -v nextflow >/dev/null 2>&1; then
		NEXTFLOW="$(command -v nextflow)"
		export NEXTFLOW
		return
	fi
	local cache="${HOME}/.cache/plant-genome-portal/nextflow"
	local bin="${cache}/nextflow-${NEXTFLOW_VERSION}"
	if [ ! -x "${bin}" ]; then
		mkdir -p "${cache}"
		echo "Downloading Nextflow ${NEXTFLOW_VERSION} to ${bin}"
		curl -fsSL \
			"https://github.com/nextflow-io/nextflow/releases/download/v${NEXTFLOW_VERSION}/nextflow-${NEXTFLOW_VERSION}-dist" \
			-o "${bin}"
		chmod +x "${bin}"
	fi
	NEXTFLOW="${bin}"
	export NEXTFLOW
}
