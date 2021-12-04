#!/usr/bin/env bash

mkdir -p sample_data/keys

# Generate a new private key
if ! [ -f "sample_data/keys/ED25519_private.pem" ]; then
  openssl genpkey -algorithm ED25519 -out "sample_data/keys/ED25519_private.pem"
fi

# Generate a public key from the private key.
if ! [ -f "sample_data/keys/ED25519_public.pem" ]; then
  openssl pkey -in "sample_data/keys/ED25519_private.pem" -pubout -out "sample_data/keys/ED25519_public.pem"
fi