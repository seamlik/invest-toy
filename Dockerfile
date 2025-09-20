FROM ubuntu:24.04

LABEL org.opencontainers.image.title="Builder image for project `invest-toy`"

RUN apt-get update

# Install APT packages directly required by the build
RUN apt-get install --yes git ninja-build unzip build-essential

# Install Rust
ADD https://sh.rustup.rs /opt/rustup-init.sh
RUN apt-get install --yes curl
RUN cat /opt/rustup-init.sh | bash -s -- -y
ENV PATH="$PATH:/root/.cargo/bin"

# Install PowerShell
ADD https://packages.microsoft.com/config/ubuntu/24.04/packages-microsoft-prod.deb /opt/packages-microsoft-prod.deb
RUN dpkg --install /opt/packages-microsoft-prod.deb
RUN apt-get update
RUN apt-get install --yes powershell

# Install Node.js
ADD https://deb.nodesource.com/setup_24.x /opt/nodesource-init.sh
RUN bash /opt/nodesource-init.sh
RUN apt-get install --yes nodejs

# Install from NPM
RUN npm install --global prettier quicktype typescript

# Install Deno
RUN curl -fsSL https://deno.land/install.sh | sh
ENV PATH="$PATH:/root/.deno/bin"

ENV RUSTFLAGS="--deny warnings"
