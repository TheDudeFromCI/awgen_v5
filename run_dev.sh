#!/bin/bash

echo "Running Awgen in development mode."
mkdir -p ./templates/default/assets

cargo run --features editor -- --debug --project ./templates/default $@ 2>&1 | tee latest.log
