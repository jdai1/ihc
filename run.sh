#!/bin/bash

########################################
############# CSCI 2951-O ##############
########################################
E_BADARGS=65
if [ $# -ne 1 ]
then
	echo "Usage: `basename $0` <input>"
	exit $E_BADARGS
fi

# For Mac M1 Env; Assumes CPLEX installation from cs2951o (i.e. x86 binary)

input=$1
cd src/rust
export LD_LIBRARY_PATH=/course/cs2951o/ilog/CPLEX_Studio2211/cplex/bin/x86-64_linux
export CPLEX_PATH=/course/cs2951o/ilog/CPLEX_Studio2211/cplex
cargo run --release -- $1
cd ../..