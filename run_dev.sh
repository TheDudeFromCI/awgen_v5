#!/bin/bash

echo "Running Awgen in development mode."
mkdir -p ./test_project
mkdir -p ./test_project/assets

cp -r ./assets/* ./test_project/assets

cargo run --features editor -- --debug --project ./test_project
