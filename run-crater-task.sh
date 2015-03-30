#!/bin/sh

set -u -e

say() {
    echo
    echo "# $1"
    echo
}

main() {
    local task_type="${CRATER_TASK_TYPE-}"
    if [ -z "$task_type" ]; then
	say "CRATER_TASK_TYPE not defined"
	exit 1
    fi

    if [ "$task_type" = "crate-build" ]; then
	local rust_installer="${CRATER_RUST_INSTALLER-}"
	local cargo_installer="${CRATER_CARGO_INSTALLER-}"
	local crate_file="${CRATER_CRATE_FILE-}"

	if [ -z "$rust_installer" ]; then
	    say "CRATER_RUST_INSTALLER not defined"
	    exit 1
	fi

	if [ -z "$crate_file" ]; then
	    say "CRATER_CRATE_FILE not defined"
	    exit 1
	fi

	say "Installing system packages"
	apt-get install build-essential -y

	say "Installing various native libs"
	# As a temporary convience to make various popular libraries build
	apt-get install libz-dev -y

	say "Installing Rust from $rust_installer"
	curl -Lf "$rust_installer" -o installer.tar.gz
	mkdir ./rust-install
	tar xzf installer.tar.gz -C ./rust-install --strip-components=1
	./rust-install/install.sh

	if [ -n "$cargo_installer" ]; then
	    say "Installing Cargo from $cargo_installer"
	    curl -Lf "$cargo_installer" -o cargo-installer.tar.gz
	    mkdir ./cargo-install
	    tar xzf cargo-installer.tar.gz -C ./cargo-install --strip-components=1
	    ./cargo-install/install.sh
	fi

	say "Printing toolchain versions"

	rustc --version
	cargo --version

	say "Downloading crate from $crate_file"
	curl -fL "$crate_file" -o crate.tar.gz
	mkdir ./crate
	tar xzf crate.tar.gz -C ./crate --strip-components=1

	say "Replacing path dependencies in Cargo.toml"
	if [ -e ./crate/Cargo.toml ]; then
	    # Replaces any line beginning with 'path' with an empty line, if that line
	    # occurs inside a [dependencies.*] section
	    sed -i '/\[dependencies.*\]/,/\[.*\]/ s/^\w*path.*//' ./crate/Cargo.toml
	else
	    say "Cargo.toml does not exist!"
	fi

	say "Building and testing"
	(cd ./crate && cargo build)
	# FIXME
	#(cd ./crate && cargo test)
    elif [ "$task_type" = "custom-build" ]; then
	local git_repo="${CRATER_TOOLCHAIN_GIT_REPO-}"
	local commit_sha="${CRATER_TOOLCHAIN_GIT_SHA-}"

	if [ -z "$git_repo" ]; then
	    say "CRATER_TOOLCHAIN_GIT_REPO not defined"
	    exit 1
	fi

	if [ -z "$commit_sha" ]; then
	    say "CRATER_TOOLCHAIN_GIT_SHA not defined"
	    exit 1
	fi

	say "Installing system packages"
	apt-get install build-essential -y
	apt-get install git file python2.7 -y
	apt-get install -y build-essential python perl curl git libc6-dev-i386 gcc-multilib g++-multilib llvm llvm-dev
	apt-get build-dep -y clang llvm

	say "Cloning git repo"
	git clone "$git_repo" rust && (cd rust && git reset "$commit_sha" --hard)

	say "Configuring"
	(cd rust && ./configure --build=x86_64-unknown-linux-gnu --host=x86_64-unknown-linux-gnu --target=x86_64-unknown-linux-gnu)

	say "Building"
	(cd rust && make && make dist)

	say "Renaming installer"
	mv rust/dist/rustc-*-x86_64-unknown-linux-gnu.tar.gz \
           rust/dist/rustc-dev-x86_64-unknown-linux-gnu.tar.gz

    else
	say "Unknown task type"
	exit 1
    fi
}

main "$@"

