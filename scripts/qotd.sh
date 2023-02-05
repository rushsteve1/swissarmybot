#!/usr/bin/env bash
#
# Quote of the Day (QOTD)
# Run with cron or a systemd timer
# 0 5 * * *

curl -X POST -d "**Quote of the Day**" 'https://elfnein.rushsteve1.us/bot/random?channel=421471250733465610'
