#!/bin/sh

nodemon -w . -e ".rs",".js",".html" -x "source local.sh && cargo run"
