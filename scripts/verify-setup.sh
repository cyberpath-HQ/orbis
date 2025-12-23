#!/bin/bash

# Orbis Development Environment Verification Script
# This script checks if your system is properly configured for Orbis development

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Track results
PASSED=0
FAILED=0
WARNINGS=0

# Utility functions
print_header() {
    echo -e "\n${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

print_check() {
    echo -e "${CYAN}▸${NC} $1"
}

pass() {
    echo -e "  ${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "  ${RED}✗${NC} $1"
    ((FAILED++))
}

warn() {
    echo -e "  ${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

info() {
    echo -e "  ${BLUE}ℹ${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            DISTRO=$ID
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        OS="windows"
    else
        OS="unknown"
    fi
}

# Check Rust
check_rust() {
    print_header "Rust Toolchain"
    
    print_check "Rust installation"
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        pass "$RUST_VERSION"
    else
        fail "Rust not found. Install from https://rustup.rs/"
        return 0
    fi
    
    print_check "Cargo"
    if command -v cargo &> /dev/null; then
        CARGO_VERSION=$(cargo --version)
        pass "$CARGO_VERSION"
    else
        fail "Cargo not found"
        return 0
    fi
    
    print_check "WASM target (wasm32-unknown-unknown)"
    if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
        pass "wasm32-unknown-unknown installed"
    else
        warn "wasm32-unknown-unknown not installed. Installing now..."
        if rustup target add wasm32-unknown-unknown 2>/dev/null; then
            pass "wasm32-unknown-unknown installed"
        else
            fail "Failed to install wasm32-unknown-unknown"
        fi
    fi
    
    print_check "Rust nightly"
    if rustup toolchain list | grep -q "nightly"; then
        NIGHTLY_VERSION=$(rustc +nightly --version 2>/dev/null || echo "nightly")
        info "$NIGHTLY_VERSION"
    else
        warn "Rust nightly not installed (optional, only needed for certain features)"
    fi
}

# Check Node.js and Bun
check_node() {
    print_header "Node.js & Package Managers"
    
    print_check "Node.js"
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version)
        NODE_MAJOR=$(echo $NODE_VERSION | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$NODE_MAJOR" -ge 20 ]; then
            pass "$NODE_VERSION"
        else
            fail "Node.js 20+ required, found $NODE_VERSION"
            return 0
        fi
    else
        fail "Node.js not found. Install from https://nodejs.org/"
        return 0
    fi
    
    print_check "npm"
    if command -v npm &> /dev/null; then
        NPM_VERSION=$(npm --version)
        pass "npm $NPM_VERSION"
    else
        fail "npm not found"
        return 0
    fi
    
    print_check "Bun (recommended)"
    if command -v bun &> /dev/null; then
        BUN_VERSION=$(bun --version)
        pass "Bun $BUN_VERSION"
    else
        warn "Bun not installed. Install from https://bun.sh/ for better performance"
    fi
}

# Check Git
check_git() {
    print_header "Version Control"
    
    print_check "Git"
    if command -v git &> /dev/null; then
        GIT_VERSION=$(git --version)
        pass "$GIT_VERSION"
    else
        fail "Git not found. Install from https://git-scm.com/"
        return 0
    fi
}

# Check system dependencies based on OS
check_system_deps() {
    print_header "System Dependencies"
    
    if [ "$OS" == "linux" ]; then
        check_linux_deps
    elif [ "$OS" == "macos" ]; then
        check_macos_deps
    elif [ "$OS" == "windows" ]; then
        check_windows_deps
    else
        warn "Unknown operating system, skipping OS-specific checks"
    fi
}

check_linux_deps() {
    print_check "Checking Linux dependencies"
    
    local missing_deps=()
    
    # Check for required packages
    if ! dpkg -l | grep -q libwebkit2gtk-4.1-dev 2>/dev/null; then
        if ! rpm -q webkit2gtk4.1-devel &>/dev/null; then
            missing_deps+=("libwebkit2gtk-4.1-dev")
        fi
    else
        info "libwebkit2gtk-4.1-dev found"
    fi
    
    if ! dpkg -l | grep -q build-essential 2>/dev/null; then
        if ! rpm -q gcc &>/dev/null; then
            missing_deps+=("build-essential")
        fi
    else
        info "build-essential found"
    fi
    
    if ! dpkg -l | grep -q libssl-dev 2>/dev/null; then
        if ! rpm -q openssl-devel &>/dev/null; then
            missing_deps+=("libssl-dev")
        fi
    else
        info "libssl-dev found"
    fi
    
    if [ ${#missing_deps[@]} -eq 0 ]; then
        pass "All required Linux dependencies installed"
    else
        warn "Missing some system dependencies: ${missing_deps[*]}"
        if command -v apt &> /dev/null; then
            info "Install with: sudo apt install ${missing_deps[*]}"
        elif command -v dnf &> /dev/null; then
            info "Install with: sudo dnf install ${missing_deps[*]}"
        elif command -v pacman &> /dev/null; then
            info "Install with: sudo pacman -S ${missing_deps[*]}"
        fi
    fi
}

check_macos_deps() {
    print_check "Checking macOS dependencies"
    
    if xcode-select -p &>/dev/null; then
        pass "Xcode Command Line Tools installed"
    else
        warn "Xcode Command Line Tools not found. Install with: xcode-select --install"
    fi
}

check_windows_deps() {
    print_check "Checking Windows dependencies"
    
    if command -v cl &> /dev/null; then
        pass "Visual C++ compiler found"
    else
        warn "Visual C++ compiler not found. Install Visual Studio Build Tools from https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    fi
}

# Check repository structure
check_repo_structure() {
    print_header "Repository Structure"
    
    local dirs=("crates" "orbis" "plugins" "docs")
    
    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ]; then
            pass "Directory: $dir"
        else
            fail "Missing directory: $dir"
        fi
    done
}

# Check dependency installation
check_dependencies_installed() {
    print_header "Project Dependencies"
    
    if [ -d "orbis/node_modules" ]; then
        pass "orbis/node_modules installed"
    else
        warn "orbis/node_modules not found. Run: cd orbis && bun install"
    fi
    
    if [ -d "docs/node_modules" ]; then
        pass "docs/node_modules installed (optional)"
    else
        info "docs/node_modules not found (optional). Run: cd docs && bun install"
    fi
}

# Print summary
print_summary() {
    print_header "Summary"
    
    local total=$((PASSED + FAILED + WARNINGS))
    
    echo -e "${GREEN}✓ Passed: $PASSED${NC}"
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}⚠ Warnings: $WARNINGS${NC}"
    fi
    if [ $FAILED -gt 0 ]; then
        echo -e "${RED}✗ Failed: $FAILED${NC}"
    fi
    
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        if [ $WARNINGS -eq 0 ]; then
            echo -e "${GREEN}${BOLD}🎉 All checks passed! You're ready to develop with Orbis!${NC}"
            echo ""
            echo -e "Next steps:"
            echo -e "  1. ${CYAN}cd orbis${NC}"
            echo -e "  2. ${CYAN}bun run tauri dev${NC} - Start development server"
            echo -e "  3. Check the ${CYAN}README.md${NC} for more information"
        else
            echo -e "${YELLOW}${BOLD}⚠ Most checks passed, but there are some warnings.${NC}"
            echo -e "Review the warnings above and install any recommended tools."
        fi
    else
        echo -e "${RED}${BOLD}✗ Setup incomplete! Please fix the failed checks above.${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "For more help, visit: ${CYAN}https://github.com/cyberpath-HQ/orbis${NC}"
    echo ""
}

# Main execution
main() {
    echo -e "${BOLD}${GREEN}"
    echo "  ____  ___  ___  ________"
    echo " / __ \/ _ \/ _ )/  _/ __/"
    echo "/ /_/ / , _/ _  |/ /_\ \  "
    echo "\____/_/|_/____/___/___/  "
    echo ""
    echo -e "${NC}${BOLD}Development Environment Verification${NC}"
    echo -e "Version: 1.0.0 | Updated: 2025-12-22\n"
    
    detect_os
    info "Detected OS: $OS"
    [ -n "$DISTRO" ] && info "Distribution: $DISTRO"
    echo ""
    
    # Run all checks
    check_rust
    check_node
    check_git
    check_system_deps
    check_repo_structure
    check_dependencies_installed
    
    # Print summary
    print_summary
}

# Run main function
main "$@"
