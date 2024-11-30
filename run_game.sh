#!/bin/bash

echo "Running Awgen in gameplay mode."
mkdir -p ./templates/default

cargo run -- --project ./templates/default --fullscreen $@ 2>&1 | tee latest.log
