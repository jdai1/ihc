# export DYLD_LIBRARY_PATH=/Applications/CPLEX_Studio2211/cplex/bin/x86-64_osx 
# export CPLEX_PATH=/Applications/CPLEX_Studio2211/cplex 
CPLEX_PATH=/Applications/CPLEX_Studio2211/cplex DYLD_LIBRARY_PATH=/Applications/CPLEX_Studio2211/cplex/bin/x86-64_osx cargo run --release --target x86_64-apple-darwin -- $1