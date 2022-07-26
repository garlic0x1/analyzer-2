#!/bin/bash
#docker build -t analyzer .
while read line; do echo $line | hakrawler -i -d 4 | grep ".php" | xargs -I %s printf "%s " | tee | docker run -i --rm analyzer | tee | webhook ; done < trunks.txt

