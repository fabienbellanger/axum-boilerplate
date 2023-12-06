#!/bin/bash

cd "$(dirname "$0")" || exit
target/debug/./axum-boilerplate-bin serve
