#!/bin/bash

set -eoux

ulimit -s unlimited
./test_verify_for_guest /mnt/input.json output.wtns
rapidsnark test_verify_for_guest_final.zkey output.wtns /mnt/proof.json /mnt/public.json