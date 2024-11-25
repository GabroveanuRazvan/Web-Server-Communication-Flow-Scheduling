#!/bin/bash

find /tmp/tmpfs -maxdepth 1 -type d -name 'cache*' -exec rm -rf {} +