#!/bin/bash
npx @openapitools/openapi-generator-cli generate -i sonarr-openapi-v3.yaml -g rust -o sonarr-sdk --additional-properties=packageName=sonarr
