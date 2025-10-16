#!/bin/bash -xe

# Fix ownership of volume mounted directories
sudo chown vscode:vscode /home/vscode/.aws /usr/local/cargo/registry /usr/local/cargo/git /workspaces/archodex-backend/target

# Install / update SurrealDB CLI
curl -sSf https://install.surrealdb.com | sudo sh