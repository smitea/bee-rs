#!/usr/bin/env bash
target=$1

cargo install cross
cross build --target $target --release

mkdir bin

if [[ $1 == x86_64-pc-windows* ]] 
then
    cp target/$1/release/bee.exe bin/
    cp target/$1/release/hive.exe bin/
    cp install.vbs bin/
    cp uninstall.vbs bin/
else
    cp target/$1/release/bee bin/
    cp target/$1/release/hive bin/
fi

tar -zcvf bee.$1.tar.gz bin
rm -rf bin

