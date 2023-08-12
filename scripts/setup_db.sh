#!/usr/bin/env sh

set -e

name="manga"
port=5436

docker container inspect $name >/dev/null 2>&1 || docker run -d --restart unless-stopped --name $name -p $port:5432 -e POSTGRES_PASSWORD=password postgres:latest >/dev/null

echo 'export DATABASE_URL="postgresql://postgres:password@localhost:'$port'/postgres"'
