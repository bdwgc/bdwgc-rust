#!/bin/sh

set -ex

duration=20

cd $(dirname $0)

for directory in $(ls examples); do
  (
    cd examples/$directory
    cargo run &
    pid=$!
    sleep $duration
    kill $pid
  )
done
