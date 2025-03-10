FROM ubuntu:24.10

LABEL org.opencontainers.image.title="Builder image for project `invest-toy`"

RUN apt-get update

# Install APT packages directly required by the build
RUN apt-get install --yes git ninja-build

# Install Rust
ADD https://sh.rustup.rs /opt/rustup-init.sh
RUN apt-get install --yes curl
RUN cat /opt/rustup-init.sh | bash -s -- -y
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup target add wasm32-unknown-unknown

# Install from crates.io
RUN apt-get install --yes build-essential
RUN cargo install resvg wasm-pack

# Install PowerShell
ADD https://packages.microsoft.com/config/ubuntu/24.04/packages-microsoft-prod.deb /opt/packages-microsoft-prod.deb
RUN dpkg --install /opt/packages-microsoft-prod.deb
RUN apt-get update
RUN apt-get install --yes powershell

# Install Node.js
ADD https://deb.nodesource.com/setup_23.x /opt/nodesource-init.sh
RUN bash /opt/nodesource-init.sh
RUN apt-get install --yes nodejs

# Install from NPM
RUN npm install --global esbuild eslint prettier quicktype typescript

# Install Binaryen
ADD https://github.com/WebAssembly/binaryen/releases/download/version_122/binaryen-version_122-x86_64-linux.tar.gz /opt/binaryen.tar.gz
RUN mkdir /opt/binaryen
RUN tar --extract --gzip --strip-components=1 --file=/opt/binaryen.tar.gz --directory=/opt/binaryen
ENV PATH="$PATH:/opt/binaryen/bin"

ENV RUSTFLAGS="--deny warnings"
