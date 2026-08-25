FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git \
        ca-certificates \
        build-essential \
    && rm -rf /var/lib/apt/lists/*
