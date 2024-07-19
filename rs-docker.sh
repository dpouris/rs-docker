#!/bin/bash

docker compose run --rm --cap-add="SYS_ADMIN" helper "$@"
