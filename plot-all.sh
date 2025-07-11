#!/bin/bash
set -x
python3 plot.py exp-data.txt exp-improve.eps
python3 plot.py hyper-data.txt hyper-improve.eps 0.01
python3 plot.py uniform-data.txt uniform-improve.eps
