#!/bin/bash

set -eoux

ulimit -s unlimited
./verify_for_guest /mnt/input.json output.wtns
rapidsnark verify_for_guest_final.zkey output.wtns /mnt/proof.json /mnt/public.json