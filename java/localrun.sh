#!/bin/bash

########################################
############# CSCI 2951-O ##############
########################################

#Example of how to run on a M1 Mac locally
E_BADARGS=65
if [ $# -ne 1 ]
then
	echo "Usage: `basename $0` <input>"
	exit $E_BADARGS
fi
	
input=$1
# export the solver libraries into the path
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/Applications/CPLEX_Studio2211/cpoptimizer/bin/x86-64_osx:/Applications/CPLEX_Studio2211/cplex/bin/x86-64_osx

# Set x86 java
x86java="/Applications/CPLEX_Studio2211/opl/oplide/jdk-18.0.2+9-jre/Contents/Home/bin/java"
$x86java -Djava.library.path="/Applications/CPLEX_Studio2211/opl/bin/x86-64_osx" -cp /Applications/CPLEX_Studio2211//cplex/lib/cplex.jar:src solver.ip.Main $input
