#!/bin/bash

cd /tmp && mkdir tmpfs && sudo mount -t tmpfs -o size=100M tmpfs /tmp/tmpfs
