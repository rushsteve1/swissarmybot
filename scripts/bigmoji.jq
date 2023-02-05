# Run with -r
.bigmoji | to_entries | sort_by(.key)[] | [.key, .value, (now | todate)] | @csv
