#!/usr/bin/env bash
#
# Daily Stonks Report
# Run with cron or a systemd timer
# 0 17 * * *

curl -X POST 'https://elfnein.rushsteve1.us/bot/stonks?channel=859531364906172436'
