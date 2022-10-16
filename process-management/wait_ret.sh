#!/bin/bash
false &
wait &! # wait for false process
echo "finsh false command: $?"
