# Run with -r
.quotes | to_entries | map(.value) | sort_by(.time)[] | [.text, .user_id, "user name", .adder_id, "adder name", (.time | todate)] | @csv
