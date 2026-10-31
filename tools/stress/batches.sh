#!/bin/bash
L=$(dirname "$0")
D=${DUR:-720}
$L/run.sh b1 --versions "1.39,1.41r1,2.05,2.22" --per-version 2 --duration $D --churn 0.15 --seed 11
$L/run.sh b2 --versions "5.00r7,5.12,5.24r4" --per-version 2 --duration $D --churn 0.15 --seed 12
$L/run.sh b3 --versions "5.30r3,5.31r4,6.00r2" --per-version 2 --duration $D --churn 0.15 --seed 13
$L/run.sh b4 --versions "6.015,6.034,6.03r7,6.03r7 Linux" --per-version 2 --duration $D --churn 0.15 --seed 14
echo ALL DONE
