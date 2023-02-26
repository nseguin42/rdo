#!/bin/bash
if [ -z "$2" ]; then
    echo "$1"
else
  echo "$1 | $2" && read
fi