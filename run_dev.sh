#!/bin/bash

echo "Running Awgen in development mode."
mkdir -p ./test_project/assets

cargo run --features editor -- --debug --project ./test_project $@ 2>&1 | tee ./test_project/latest.log
