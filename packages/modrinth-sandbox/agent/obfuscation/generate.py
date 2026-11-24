#!/bin/python3

import os
import tarfile
from compression import zstd

method = input("Intermediary method name: ")

if not method.startswith("method_"):
    print("Expected method to start with method_")
    exit(1)

data = []

# The archive is compressed with `zstd --long=31`, so the decompressor must allow a 2 GiB window
mappings_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "mappings.tar.zst")
zstd_options = {zstd.DecompressionParameter.window_log_max: 31}
with zstd.ZstdFile(mappings_path, options=zstd_options) as compressed, tarfile.open(fileobj=compressed, mode="r|") as mappings:
    for member in mappings:
        filename = member.name
        if not member.isfile() or not filename.endswith(".tiny"):
            continue
        contents = mappings.extractfile(member).read().decode("utf-8")
        for line in contents.splitlines():
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
