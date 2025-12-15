#!/bin/sh

set -ex

cd $(dirname $0)

for directory in $(ls examples); do
  (
    cd examples/$directory
    cargo run &
    pid=$!
    sleep 20
    kill $pid
  )
done
