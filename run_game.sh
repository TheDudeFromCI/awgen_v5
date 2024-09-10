#!/bin/bash

echo "Running Awgen in gameplay mode."
mkdir -p ./test_project

cargo run -- --project ./test_project
