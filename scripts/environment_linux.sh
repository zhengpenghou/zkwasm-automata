#!/bin/bash
# environments setup

check_mongoDB_installed() {
    echo "Check if mongoDB installed."
    if [[ ( ! -f "`which mongod`" ) ]]
    then
        curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | \
            sudo gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg \
            --dearmor
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] \
            https://repo.mongodb.org/apt/ubuntu $(lsb_release -cs)/mongodb-org/7.0 multiverse" | \
            sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list
        sudo apt update
        sudo apt install -y mongodb-org
    fi

    if [ -f "`which mongod`" ]; then
        echo "mongoDB is ready ✅"
    else
        echo "mongoDB install failed ❌, please install it manually."
    fi
}

check_redis_installed() {
    echo "Check if Redis installed."
    if [[ ( ! -f "`which redis-server`" ) ]]
    then
        sudo add-apt-repository ppa:redislabs/redis -y
        sudo apt update
        sudo apt install -y redis
    fi

    if [ -f "`which redis-server`" ]; then
        echo "Redis is ready ✅"
    else
        echo "Redis install failed ❌, please install it manually."
    fi
}

check_rustup_installed() {
    echo "Check if rustup is installed ..."
    if [ ! -f "`which rustup`" ]; then
        echo "rustup not found, perform install";
        curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh;
        . $HOME/.cargo/env;
    fi

    if [ -f "`which rustup`" ]; then
        echo "rustup is ready ✅"
    else
        echo "rustup install failed ❌, please install it manually."
    fi
}

check_wasm-pack_installed() {
    echo "Check if wasm-pack is installed ..."
    if [ ! -f "`which wasm-pack`" ]; then
	echo "wasm-pack not found, perform install";
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi

    if [ -f "`which wasm-pack`" ]; then
        echo "wasm-pack is ready ✅"
    else
        echo "wasm-pack install failed ❌, please install it manually."
    fi
}

check_wasm-opt_installed() {
    echo "Check if wasm-opt is installed ..."
    if [ ! -f "`which wasm-opt`" ]; then
        echo "wasm-opt not found, perform install"
        echo "Downloading binaryen..."
        curl -L https://github.com/WebAssembly/binaryen/releases/download/version_118/binaryen-version_118-x86_64-linux.tar.gz \
            -o binaryen.tar.gz
        echo "Extracting wasm-opt..."
        tar -xzf binaryen.tar.gz
        echo "Installing wasm-opt..."
        sudo mv binaryen-version_118/bin/wasm-opt /usr/local/bin/
        echo "Cleaning up..."
        rm -rf binaryen.tar.gz binaryen-version_118
    fi

    if [ -f "`which wasm-opt`" ]; then
        echo "wasm-opt is ready ✅"
    else
        echo "wasm-opt install failed ❌, please install it manually."
    fi
}

check_nodejs_installed() {
    echo "Check if nodejs is installed ..."
    if [ ! -f "`which node`" ]; then
        echo "nodejs not found, perform install";
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
        . $HOME/.nvm/nvm.sh
        nvm install 20
    fi

    if [ -f "`which node`" ]; then
        echo "nodejs is ready ✅"
    else
        echo "nodejs install failed ❌, please install it manually."
    fi
}

echo "Prepare environment ..."

echo "1>. Create environment.tmp folder."
[ -d "./environment.tmp" ] || mkdir environment.tmp

echo "2>. Install build tools ..."
cd environment.tmp
sudo apt update
sudo apt install -y gnupg curl git cmake build-essential
check_mongoDB_installed
check_redis_installed
check_rustup_installed
check_wasm-pack_installed
check_wasm-opt_installed
check_nodejs_installed

echo "3>. Cleanup environment.tmp folder."
cd ..
[ -d "./environment.tmp" ] && rm -rf environment.tmp
echo "Environment is setup successfully. 💯"