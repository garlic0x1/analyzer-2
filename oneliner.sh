#!/bin/bash
while read line; do echo $line | hakrawler -i -d 4 | grep ".php" | xargs -I %s printf "%s " | tee |  ./target/debug/analyzer | webhook ; done < trunks.txt

