#!/bin/bash

set -u

wsm="./strings2offsets.wasm"

out='./out.offsets.le.bin'

input1(){
  echo helo
  echo world
  echo hi
}

run_wasi(){
  cat /dev/stdin |
    wasmtime run "${wsm}"
}

echo -- generating the offsets... --
input1 | run_wasi > "${out}"
echo

echo -- the offsets --
# 5, 11, 14
cat "${out}" | od -t d4
echo

o0=5
o1=11
o2=14

echo -- getting the 2nd string --
input1 | dd bs=1 skip=$o0 count=$(( $o1 - $o0 )) status=none
echo

echo -- getting the last string --
input1 | dd bs=1 skip=$o1 count=$(( $o2 - $o1 )) status=none
echo
