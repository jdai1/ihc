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
export DYLD_LIBRARY_PATH=/Applications/CPLEX_Studio2211/cplex/bin/x86-64_osx 
export CPLEX_PATH=/Applications/CPLEX_Studio2211/cplex 
cargo run --release --target x86_64-apple-darwin -- $1
