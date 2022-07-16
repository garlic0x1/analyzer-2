#!/bin/bash
while read line; do echo $line | hakrawler -i -d 3 | grep ".php" | xargs -I %s printf "%s " | tee |  ./target/debug/analyzer ; done < trunks.txt

