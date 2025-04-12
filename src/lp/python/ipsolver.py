from typing import List
import math

from lpsolver import LPSolver
import heapq
import numpy as np

def is_integral_assignments(assignments: list):
    for x in assignments:
        if not isinstance(x, int) and not x.is_integer():
            return False
    return True


""" Idea: Best-First Search with heuristic that weights cost + # variables assigned """

class DFSIPSolver:
    def __init__(self, filename: str):
        self.num_tests, self.num_diseases = self.data_parse(filename)  # Optional if needed later
        self.stack: List[int] = []
        self.trail: List[int] = []
        self.assignments: List[int] = [-1] * self.num_tests
        self.lp_solver = LPSolver(filename=filename)
        self.incumbent_cost: float = math.inf
        self.incumbent_assignment: List[float] = [0.0] * self.num_tests

    def data_parse(self, filename: str):
        try:
            with open(filename, "r") as fl:
                numTests = int(fl.readline().strip())  # n
                numDiseases = int(fl.readline().strip())  # m

                return numTests, numDiseases
        except Exception as e:
            print(f"Error reading instance file. File format may be incorrect.{e}")
            exit(1)

    def solve(self) -> float:
        branch_var = self.get_branch_var()
        self.stack.append(-branch_var)
        self.stack.append(branch_var)

        while self.stack:
            next_var = self.stack.pop()
            self.assign(next_var)
            feasible, objective_value, assignments = self.lp_solver.solve(self.assignments)

            if not feasible:
                # print("Backtracking b/c infeasible")
                self.backtrack()
            elif objective_value > self.incumbent_cost:
                # print("Backtracking b/c prunable branch")
                self.backtrack()
            elif is_integral_assignments(assignments):
                # print("Backtracking b/c all integral assignments")
                if objective_value < self.incumbent_cost:
                    # print(f"New incumbent: {objective_value}")
                    self.incumbent_cost = objective_value
                    self.incumbent_assignment = assignments
                self.backtrack()
            else:
                branch_var = self.get_branch_var()
                if branch_var == 0:
                    # print("Backtracking b/c all vars have been assigned")
                    self.backtrack()
                else:
                    self.stack.append(-branch_var)
                    self.stack.append(branch_var)

        return self.incumbent_cost

    def assign(self, var: int):
        self.assignments[abs(var) - 1] = 1 if var > 0 else 0
        self.trail.append(var)

    def get_branch_var(self) -> int:
        for i, val in enumerate(self.assignments):
            if val == -1:
                return i + 1
        return 0

    def backtrack(self):
        if not self.stack:
            return
        next_var = self.stack[-1]
        while self.trail:
            var = abs(self.trail.pop())
            self.assignments[var - 1] = -1
            if var == abs(next_var):
                break


class BFSIPSolver:
    def __init__(self, filename: str):
        self.num_tests, self.num_diseases, self.cost, self.A = self.data_parse(filename)  # Optional if needed later
        # ((cost, assignment, diff_diseases) pairs for BeFS)
        self.heap: List[tuple[float, list[int], np.ndarray]] = []
        self.lp_solver = LPSolver(filename=filename)
        self.incumbent_cost: float = math.inf
        self.incumbent_assignment: List[float] = [0.0] * self.num_tests

        # pre-processing
        table = []
        for i in range(self.num_diseases):
            for j in range(i + 1, self.num_diseases):
                table.append((self.A[:, i] - self.A[:, j]) ** 2)
        self.table = np.stack(table, axis=1).astype(np.uint8)

        # sort by efficacy
        # TODO: bitmaps!
        # 1. calculate bitmaps for each test
        # 2. (in solve?) maintain current mask of differentiated diseases
        # 3. 
        # l = [(np.sum(table[i, :]), i) for i in range(self.num_tests)]
        # l.sort(key=lambda x: x[0], reverse=True)
        # self.sorted_tests = [x[1] for x in l]

    def data_parse(self, filename: str):
        try:
            with open(filename, "r") as fl:
                numTests = int(fl.readline().strip())  # n
                numDiseases = int(fl.readline().strip())  # m

                costOfTest = np.array(
                    [float(i) for i in fl.readline().strip().split()]
                )  # length numT

                A = np.zeros((numTests, numDiseases))
                for i in range(0, numTests):
                    A[i, :] = np.array(
                        [int(i) for i in fl.readline().strip().split()]
                    )  # numT x numD
                return numTests, numDiseases, costOfTest, A
        except Exception as e:
            print(f"Error reading instance file. File format may be incorrect.{e}")
            exit(1)

    def solve(self) -> float:
        branch_var = self.get_branch_var_goated([-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))
        self.search(branch_var, [-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))
        self.search(-branch_var, [-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))

        while self.heap:
            _, assignments, diff_dis = heapq.heappop(self.heap)
            branch_var = self.get_branch_var_goated(assignments, diff_dis)
            if branch_var == 0:
                continue
            self.search(branch_var, assignments[:], diff_dis.copy())
            self.search(-branch_var, assignments, diff_dis)
            
        return self.incumbent_cost

    def search(self, branch_var: int, fixed: list[int], diff_dis: np.ndarray):
        fixed[abs(branch_var) - 1] = 1 if branch_var > 0 else 0
        feasible, objective_value, lp_assignments = self.lp_solver.solve(fixed)

        if branch_var > 0:
            diff_dis = diff_dis | self.table[abs(branch_var) - 1]
        
        if not feasible:
            pass
        elif objective_value > self.incumbent_cost:
            pass 
        elif is_integral_assignments(lp_assignments):
            if objective_value < self.incumbent_cost:
                self.incumbent_cost = objective_value
                self.incumbent_assignment = lp_assignments
        else:
            heapq.heappush(self.heap, (objective_value, fixed, diff_dis))
        

    # def get_branch_var_sorted(self, assignments: list[int]) -> int:
    #     for i in self.sorted_tests:
    #         if assignments[i] == -1:
    #             return i + 1
    #     return 0
    

    def get_branch_var_sequential(self, assignments: list[int]) -> int:
        for i, val in enumerate(assignments):
            if val == -1:
                return i + 1
        return 0

    def get_branch_var_goated(self, assignments: list[int], diff_dis: np.ndarray) -> int:
        # [1, 0, -1]
        print("\n\ngetting a goated new branch var")
        print(f"current assignments -- {assignments}")
        print(f"current diff_dis -- {diff_dis}")
        unassigned = [i for i, val in enumerate(assignments) if val == -1]
        print(f"unassigned -- {unassigned}")
        print(f"table -- {self.table}")
        if not unassigned: return 0

        nondiff_dis = 1 - diff_dis

        unassigned_rows = self.table[unassigned]
        new_diffs = np.bitwise_and(unassigned_rows, nondiff_dis)
        print(f"new_diffs -- {new_diffs}")
        new_diff_counts = np.sum(new_diffs, axis=1)
        # TODO: divide by the cost

        print(f"counts -- {new_diff_counts}")

        best_i = np.argmax(new_diff_counts)
        best_var = unassigned[best_i]

        print(f"RETURNING -- {best_var + 1}")

        return best_var + 1