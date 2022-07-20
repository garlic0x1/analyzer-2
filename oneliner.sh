#!/bin/bash
while read line; do echo $line | hakrawler -i -d 4 | grep ".php" | xargs -I %s printf "%s " | tee | sudo docker run -i --rm analyzer:latest | webhook ; done < trunks.txt

