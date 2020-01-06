#!/bin/bash

set -e

privkey=~/.ssh/travis-rustdocs-rsa
enckey="id_rsa.enc"
cipher="aes-256-cbc"
SSH_KEY_TRAVIS_ID="RUSTDOCS_SSH_RSA"

sslkey=$(openssl rand -hex 32)
ssliv=$(openssl rand -hex 16)

openssl "${cipher}" -K "${sslkey}" -iv "${ssliv}" -in "${privkey}" -out "${enckey}" -e

echo "Store the following environment variables in your travis repo:"
echo "    encrypted_${SSH_KEY_TRAVIS_ID}_key    ${sslkey}"
echo "    encrypted_${SSH_KEY_TRAVIS_ID}_iv     ${ssliv}"

echo
echo "Use the following command to decrypt it:"
echo "    openssl ${cipher} -K "\${encrypted_${SSH_KEY_TRAVIS_ID}_key}\" -iv "\${encrypted_${SSH_KEY_TRAVIS_ID}_iv}\" -in \".travis/${enckey}\" -out ~/.ssh/id_rsa -d"
