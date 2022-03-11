#!/bin/bash

docker build --no-cache -t "maxwai/download-renamer-mover:$1" .
docker build -t maxwai/download-renamer-mover:latest .
docker push "maxwai/download-renamer-mover:$1"
docker push maxwai/download-renamer-mover:latest