FROM mcr.microsoft.com/devcontainers/cpp:1-debian-12

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        cmake \
        make \
        g++ \
        git \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*
