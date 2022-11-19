#!/bin/env bash

# USAGE: script <path-to-images/> <file-format> <width> <height>

PIC_DIR=$1
PIC_FORMAT=$2
OUTPUT_DIR=${PIC_DIR}resized
WIDTH=$3
HEIGHT=$4

send_request() {
  filename=$(basename $1)
  HTTP_CODE=$(curl -s --write-out "%{http_code}\n" --data-binary "@$1" \
      --output "${PIC_DIR}resized/$filename" \
      "http://localhost:8080/resize?width=${WIDTH}&height=$HEIGHT")
  if [ $HTTP_CODE != "200" ] && [ $HTTP_CODE != "000" ]
  then
    echo "FAIL: http_code=[$HTTP_CODE], file=[$filename]"
  elif [ $HTTP_CODE != "000" ] 
  then
    echo "SUCCESS: http_code=[$HTTP_CODE], file=[$filename]"
  fi
}

if [ ! -d "$OUTPUT_DIR" ] 
then
  mkdir "$OUTPUT_DIR"
fi

for file in "$PIC_DIR"*.$PIC_FORMAT
do
  if [ -f "$file" ]
  then
    send_request "$file" &
  fi
done

