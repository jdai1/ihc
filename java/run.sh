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
	
input=$1
# export the solver libraries into the path
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/local/projects/cplex/CPLEX_Studio221/cplex/bin/x86-64_linux

# add the solver jar to the classpath and run
java -cp /local/projects/cplex/CPLEX_Studio221/cplex/lib/cplex.jar:src solver.ip.Main $input
