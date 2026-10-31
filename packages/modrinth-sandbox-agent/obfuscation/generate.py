#!/bin/python3

import os

method = input("Intermediary method name: ")

if not method.startswith("method_"):
    print("Expected method to start with method_")
    exit(1)

data = []

directory = os.fsencode("mappings")
for file in os.listdir(directory):
    filename = os.fsdecode(file)
    if not filename.endswith(".tiny"):
        continue
    with open("mappings/"+filename, "r") as f:
        for line in f:
            split = line.strip().split("\t")
            if len(split) != 5:
                continue
            if split[0] == "METHOD" and split[4] == method:
                name = filename[:-5]
                data.append(name + "," + split[1] + "," + split[3] + "," + split[2])

data.sort()

with open(method+".csv", "w") as f:
    for line in data:
        f.write(line)
        f.write('\n')