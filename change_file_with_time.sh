#!/bin/bash

mkdir -p with-time
files=$(find .  -maxdepth 1 -a -name "*.jpeg")

for f in $files ; do
    timestamp=$(exiv2 pr $f|grep timestamp|grep -o "[0-9][0-9: ]*"|tr " " "_"|sed "s/://g")
    if [ ! -z "$timestamp" ];then
        mv $f with-time/IMG_$timestamp.jpg
    fi
done
