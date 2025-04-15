import json 
import sys 
from pathlib import Path
from model_timer import Timer
from lpsolver import LPSolver
from ipsolver import DFSIPSolver, BFSIPSolver
import ast

def lpsolve():
    filepath = sys.argv[1]
    assignments = ast.literal_eval(sys.argv[2])
    assignments = [-1] * 25
    watch = Timer()
    watch.start()
    solver = LPSolver(filepath)
    for i in range(100):
        feasible, objective_value, assignments = solver.solve(assignments)
        print(feasible, objective_value)
        print(" ".join([str(x) for x in assignments]))
    watch.stop()

    print(round(watch.getElapsed(), 2))

    # print("true" if feasible else "false")
    # if feasible:
    #     print(objective_value)
    #     print(" ".join([str(x) for x in assignments]))


def ipsolve():
    filepath = sys.argv[1]
    
    watch = Timer()
    watch.start()
    solver = BFSIPSolver(filepath)
    sol = solver.solve()
    watch.stop()

    res = {
        "Instance": filepath,
        "Time": round(watch.getElapsed(), 2),
        "Result": sol,
        "Solution": "OPT"
    }
    print(json.dumps(res))


def test():
    test_cases = [
        "100_100_0.25_1.ip",
        "100_100_0.5_3.ip",
        "100_200_0.5_5.ip",
        "100_50_0.5_2.ip",
        "25_25_0.5_1.ip",
        "50_100_0.5_4.ip",
        "50_25_0.5_1.ip",
        "100_100_0.25_3.ip",
        "100_100_0.5_9.ip",
        "100_200_0.5_6.ip",
        "25_25_0.25_1.ip",
        "25_50_0.5_8.ip",
        "50_100_0.5_7.ip",
        "50_50_0.5_5.ip"
    ]
    
    for test_case in test_cases:
        watch = Timer()
        watch.start()
        solver = BFSIPSolver(f"../input/{test_case}")
        sol = solver.solve()
        watch.stop()

        res = {
            "Instance": test_case,
            "Time": round(watch.getElapsed(), 2),
            "Result": sol,
            "Solution": "OPT"
        }
        print(json.dumps(res))
        
        

if __name__ == "__main__":
	# test()
    # ipsolve()
    lpsolve()
    