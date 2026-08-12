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
SKIP_COVERAGE_TOOLS="${SKIP_COVERAGE_TOOLS:-false}"

log() {
  printf '==> %s\n' "$*"
}

warn() {
  printf 'Warning: %s\n' "$*" >&2
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
  local packages=()
  configure_sudo

  if command -v apt-get >/dev/null 2>&1; then
    log "Installing build packages with apt-get"
    packages=(build-essential ca-certificates curl git libssl-dev pkg-config python3)
    if [[ "$SKIP_COVERAGE_TOOLS" != "true" ]]; then
      packages+=(lcov)
    fi
    "${SUDO[@]}" apt-get update
    "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get install -y "${packages[@]}"
  elif command -v dnf >/dev/null 2>&1; then
    log "Installing build packages with dnf"
    packages=(ca-certificates curl gcc gcc-c++ git make openssl-devel pkgconf-pkg-config python3)
    if [[ "$SKIP_COVERAGE_TOOLS" != "true" ]]; then
      packages+=(lcov)
    fi
    "${SUDO[@]}" dnf install -y "${packages[@]}"
  elif command -v yum >/dev/null 2>&1; then
    log "Installing build packages with yum"
    packages=(ca-certificates curl gcc gcc-c++ git make openssl-devel pkgconfig python3)
    if [[ "$SKIP_COVERAGE_TOOLS" != "true" ]]; then
      packages+=(lcov)
    fi
    "${SUDO[@]}" yum install -y "${packages[@]}"
  else
    echo "Unsupported Linux package manager. Install curl, git, a C/C++ compiler, pkg-config, and OpenSSL development headers manually." >&2
    exit 1
  fi
}

install_linux_docker() {
  if command -v docker >/dev/null 2>&1; then
    log "Docker CLI is already installed"
  elif command -v apt-get >/dev/null 2>&1; then
    log "Installing Docker with apt-get"
    "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get install -y docker.io
  elif command -v dnf >/dev/null 2>&1; then
    log "Installing Docker with dnf"
    "${SUDO[@]}" dnf install -y docker
  elif command -v yum >/dev/null 2>&1; then
    log "Installing Docker with yum"
    "${SUDO[@]}" yum install -y docker
  else
    echo "Docker is required for Linux server-backed tests, but it could not be installed automatically." >&2
    exit 1
  fi

  if docker info >/dev/null 2>&1; then
    return
  fi

  if command -v systemctl >/dev/null 2>&1; then
    log "Starting the Docker service"
    "${SUDO[@]}" systemctl start docker || true
  elif command -v service >/dev/null 2>&1; then
    log "Starting the Docker service"
    "${SUDO[@]}" service docker start || true
  fi

  if ! docker info >/dev/null 2>&1; then
    echo "Docker is installed, but the current user cannot access a running Docker daemon. Start Docker and ensure this user has permission to use it." >&2
    exit 1
  fi
}

install_macos_packages() {
  if ! command -v brew >/dev/null 2>&1; then
    echo "Homebrew is required on macOS. Install it from https://brew.sh and rerun this script." >&2
    exit 1
  fi

  log "Checking build packages with Homebrew"
  local build_formulae=()

  if ! brew list --versions openssl@3 >/dev/null 2>&1; then
    build_formulae+=(openssl@3)
  fi

  if ((${#build_formulae[@]} > 0)); then
    brew install "${build_formulae[@]}"
  else
    log "Homebrew build packages are already available"
  fi

  if [[ "$SKIP_COVERAGE_TOOLS" == "true" ]]; then
    log "Skipping optional coverage tools"
  elif command -v genhtml >/dev/null 2>&1; then
    log "genhtml is already available; skipping Homebrew lcov package install"
  else
    log "Installing coverage tools with Homebrew"
    if ! brew install lcov; then
      warn "Could not install the lcov package with Homebrew; HTML coverage reports will be unavailable until genhtml is installed"
    fi
  fi
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
  elif ! command -v cargo >/dev/null 2>&1 && [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi

  log "Installing Rust formatting and linting components"
  rustup toolchain install stable
  rustup default stable
  rustup component add rustfmt clippy llvm-tools-preview

  if [[ "$SKIP_COVERAGE_TOOLS" != "true" ]] && ! cargo llvm-cov --version >/dev/null 2>&1; then
    log "Installing cargo-llvm-cov"
    if ! cargo install cargo-llvm-cov --locked; then
      warn "Could not install cargo-llvm-cov; coverage reports will be unavailable"
    fi
  fi
}

verify_required_tools() {
  local missing=0
  local tool

  for tool in cargo rustc rustfmt cargo-clippy; do
    if command -v "$tool" >/dev/null 2>&1; then
      printf 'Found %-12s %s\n' "$tool" "$(command -v "$tool")"
    else
      printf 'Missing %s\n' "$tool" >&2
      missing=1
    fi
  done

  if ! cargo llvm-cov --version >/dev/null 2>&1; then
    warn "Missing cargo-llvm-cov; CODE_COV=true coverage reports will be unavailable"
  fi

  if command -v genhtml >/dev/null 2>&1; then
    printf 'Found %-12s %s\n' "genhtml" "$(command -v genhtml)"
  else
    warn "Missing genhtml; CODE_COV=true HTML coverage reports will be unavailable"
  fi

  if [[ "$missing" -ne 0 ]]; then
    echo "One or more required tools could not be installed." >&2
    exit 1
  fi

  if [[ "$(uname -s)" == "Linux" ]]; then
    if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
      printf 'Found %-12s %s\n' "docker" "$(command -v docker)"
    else
      echo "Docker is required for Linux server-backed tests." >&2
      missing=1
    fi
  else
    log "Docker is not required for macOS compile and non-server tests"
  fi

  if [[ "$(uname -s)" == "Linux" ]] && ! command -v python3 >/dev/null 2>&1; then
    echo "Missing python3, which is required by the Linux Milvus test launcher." >&2
    missing=1
  fi

  if [[ "$missing" -ne 0 ]]; then
    echo "One or more required tools could not be installed." >&2
    exit 1
  fi
}

install_system_packages
install_rust
if [[ "$(uname -s)" == "Linux" ]]; then
  install_linux_docker
fi
verify_required_tools

log "Milvus Rust SDK dependencies are installed"
