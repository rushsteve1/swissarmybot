#!/usr/bin/env bash

SHA=$(git rev-parse --short HEAD | xargs)

docker build -t "git.rushsteve1.us/rushsteve1/swissarmybot:latest" --build-arg GIT_VERSION=$SHA .
docker push "git.rushsteve1.us/rushsteve1/swissarmybot:latest"
