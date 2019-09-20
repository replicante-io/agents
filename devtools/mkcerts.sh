#!/bin/bash

# Check for easypki
easypki --version >/dev/null 2>/dev/null
if [[ $? -ne 0 ]]; then
  echo 'Need https://github.com/google/easypki to be in PATH'
  echo
  echo 'On Fedora install with:'
  echo '  $ sudo dnf install golang'
  echo '  $ go get github.com/google/easypki/cmd/easypki'
  exit 1
fi

# easypki configuration
export PKI_ROOT=./devtools/pki
export PKI_ORGANIZATION="Replicante Agents"
export PKI_ORGANIZATIONAL_UNIT=Development
export PKI_COUNTRY=EU

# Wipe the PKI store in case this is not the first run.
set -e
echo '==> Wiping any previous PKIs ...'
rm -rf ${PKI_ROOT}

# Generate CA, server cert and client cert.
mkdir -p ${PKI_ROOT}
echo '==> Generating CAs ...'
easypki create --private-key-size 4096 --filename ca --ca ca

echo '==> Generating Server Cert ...'
easypki create --private-key-size 4096 --ca-name ca --dns localhost server

echo '==> Generating Client Cert ...'
easypki create --private-key-size 4096 --ca-name ca --client client

# Combinde client certs into a single PEM file for clients that require a combined file.
cat "${PKI_ROOT}/ca/certs/client.crt" "${PKI_ROOT}/ca/keys/client.key" > "${PKI_ROOT}/ca/keys/client.pem"
