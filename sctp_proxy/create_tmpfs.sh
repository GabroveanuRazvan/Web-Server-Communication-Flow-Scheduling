#!/bin/bash

cd /tmp && mkdir tmpfs && sudo mount -t tmpfs -o size=500M tmpfs /tmp/tmpfs
