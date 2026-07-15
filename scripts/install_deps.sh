#!/usr/bin/env bash
# Licensed to the LF AI & Data foundation under one
# or more contributor license agreements. See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership. The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -euo pipefail

SUDO=()

log() {
  printf '==> %s\n' "$*"
}

configure_sudo() {
  if [[ "$(id -u)" -eq 0 ]]; then
    return
  fi

  if command -v sudo >/dev/null 2>&1; then
    SUDO=(sudo)
    return
  fi

  echo "Administrator privileges are required to install system packages." >&2
  exit 1
}

install_linux_packages() {
  configure_sudo

  if command -v apt-get >/dev/null 2>&1; then
    log "Installing build and coverage packages with apt-get"
    "${SUDO[@]}" apt-get update
    "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get install -y \
      build-essential ca-certificates curl git lcov libssl-dev pkg-config
  elif command -v dnf >/dev/null 2>&1; then
    log "Installing build and coverage packages with dnf"
    "${SUDO[@]}" dnf install -y \
      ca-certificates curl gcc gcc-c++ git lcov make openssl-devel pkgconf-pkg-config
  elif command -v yum >/dev/null 2>&1; then
    log "Installing build and coverage packages with yum"
    "${SUDO[@]}" yum install -y \
      ca-certificates curl gcc gcc-c++ git lcov make openssl-devel pkgconfig
  else
    echo "Unsupported Linux package manager. Install curl, git, lcov, genhtml, a C/C++ compiler, pkg-config, and OpenSSL development headers manually." >&2
    exit 1
  fi
}

install_macos_packages() {
  if ! command -v brew >/dev/null 2>&1; then
    echo "Homebrew is required on macOS. Install it from https://brew.sh and rerun this script." >&2
    exit 1
  fi

  log "Installing build and coverage packages with Homebrew"
  brew install lcov openssl@3 pkg-config
}

install_system_packages() {
  case "$(uname -s)" in
    Linux)
      install_linux_packages
      ;;
    Darwin)
      install_macos_packages
      ;;
    *)
      echo "Unsupported operating system: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

install_rust() {
  if ! command -v rustup >/dev/null 2>&1; then
    log "Installing the Rust toolchain with rustup"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi

  log "Installing Rust formatting, linting, and coverage components"
  rustup toolchain install stable
  rustup default stable
  rustup component add rustfmt clippy llvm-tools-preview

  if ! cargo llvm-cov --version >/dev/null 2>&1; then
    log "Installing cargo-llvm-cov"
    cargo install cargo-llvm-cov --locked
  fi
}

verify_required_tools() {
  local missing=0
  local tool

  for tool in cargo rustc rustfmt cargo-clippy lcov genhtml; do
    if command -v "$tool" >/dev/null 2>&1; then
      printf 'Found %-12s %s\n' "$tool" "$(command -v "$tool")"
    else
      printf 'Missing %s\n' "$tool" >&2
      missing=1
    fi
  done

  if ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "Missing cargo-llvm-cov" >&2
    missing=1
  fi

  if [[ "$missing" -ne 0 ]]; then
    echo "One or more required tools could not be installed." >&2
    exit 1
  fi

  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    printf 'Found %-12s %s\n' "docker" "$(command -v docker)"
  else
    echo "Warning: Docker with the Compose plugin is required by scripts/run_tests.sh and server-backed tests." >&2
  fi
}

install_system_packages
install_rust
verify_required_tools

log "Milvus Rust SDK dependencies are installed"
