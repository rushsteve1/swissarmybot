#!/usr/bin/env bash

rm -f sab_test.sqlite
cat migrations/*.sql | sqlite3 sab_test.sqlite