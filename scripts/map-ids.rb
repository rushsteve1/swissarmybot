#!/usr/bin/env ruby

require "json"
require "csv"

mapping = JSON.load(File.open("./mapping.json", "r"))

input = CSV.new(ARGF)
output = CSV.open("./out.csv", "w+")

input.each_with_index do |row, i|
    user = mapping[row[1]] || {}
    author = mapping[row[3]] || {}

    row[1] = user["id"] || row[1]
    row[2] = user["name"] || row[2]
    row[3] = author["id"] || row[3]
    row[4] = author["name"] || row[4]
    row.unshift(i + 1)

    output << row
end
