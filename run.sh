#!/bin/sh

jar=$(find build/libs -name "*-all.jar")

java -jar "$jar"  "$@" || echo "run error code: $?"
