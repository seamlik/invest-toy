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

ENV RUSTFLAGS="--deny warnings"
