#!/usr/bin/env bash
target=$1

cargo install cross
cross build --target $target --release

rm -rf bin && mkdir bin

cp target/$1/release/bee bin/
cp target/$1/release/hive bin/

tar -zcvf bee.$1.tar.gz bin
