#!/bin/bash

echo "Running Awgen in development mode."
mkdir -p ./test_project

cargo run --features editor -- --debug --project ./test_project
