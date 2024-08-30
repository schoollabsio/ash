#!/bin/sh

script_path="$( cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd )"

source $script_path/local.sh && $script_path/target/debug/ash
