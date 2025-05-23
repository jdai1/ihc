from typing import List
import math



from lpsolver import LPSolver
import heapq
import numpy as np
import time

def is_not_integer(x):
    return not isinstance(x, int) and not x.is_integer()

def is_integral_assignments(assignments: list):
    for x in assignments:
        if is_not_integer(x):
            return False
    return True

class colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    OKRED = '\033[91m'
    OKYELLOW = '\033[93m'
    # OKPURPLE = '\e[0;35m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

class SolveStats:
    lp_solves: int = 0                # the number of solve requests sent to the LP Solver
    leaves_pruned: int = 0            #
    leaves_integral: int = 0          #
    leaves_better_integral: int = 0
    leaves_infeasible: int = 0
    heap_solves_skipped: int = 0

    def leaves_total(self) -> int:
        return self.leaves_pruned + self.leaves_integral + self.leaves_infeasible
    
    def print(self):
        if self.leaves_total() == 0:
            print("no leaves right now, thank you, come again")
            return
        
        print("------------ SEARCH STATS ------------")
        print(f"* {self.lp_solves} lp solves ({self.heap_solves_skipped} skipped)")
        print(f"* {self.leaves_total()} total leaf nodes")
        print(f"    - {self.leaves_pruned} ({round(self.leaves_pruned * 100 / self.leaves_total(), 2)}%) were {colors.OKBLUE}pruned{colors.ENDC}")
        print(f"    - {self.leaves_infeasible} ({round(self.leaves_infeasible * 100 / self.leaves_total(), 2)}%) were {colors.OKRED}infeasible{colors.ENDC}")
        print(f"    - {self.leaves_integral} ({round(self.leaves_integral * 100 / self.leaves_total(), 2)}%) were {colors.OKGREEN}integral{colors.ENDC}")
        print(f"        - {self.leaves_better_integral} ({round(self.leaves_better_integral * 100 / self.leaves_integral, 2)}% of integral) were {colors.HEADER}new best{colors.ENDC}")



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
        self.heap: List[tuple[float, list[int], np.ndarray, list[float]]] = []
        self.lp_solver = LPSolver(filename=filename)
        self.incumbent_cost: float = math.inf
        self.incumbent_assignment: List[float] = [0.0] * self.num_tests
        self.stats = SolveStats()

        # pre-processing
        table = []
        for i in range(self.num_diseases):
            for j in range(i + 1, self.num_diseases):
                table.append((self.A[:, i] - self.A[:, j]) ** 2)
        self.table = np.stack(table, axis=1).astype(np.uint8)

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
        # branch_var = self.get_branch_var_diff_dis([-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))
        # branch_var = self.get_branch_var_fractional([-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8), [0.5] * self.num_tests)
        branch_var = self.get_branch_var_lp([-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8), [0.5] * self.num_tests)
        self.search(branch_var, [-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))
        self.search(-branch_var, [-1] * self.num_tests, np.zeros(self.table.shape[1], dtype=np.uint8))

        last_print = time.time()
        while self.heap:
            current = time.time()
            if current - last_print > 5:
                last_print = time.time()
                # print(f"heap len -- {len(self.heap)}")
                self.stats.print()
            
            parent_obj_val, assignments, diff_dis, lp_assignments = heapq.heappop(self.heap)
            # branch_var = self.get_branch_var_diff_dis(assignments, diff_dis)
            # branch_var = self.get_branch_var_fractional(assignments, diff_dis, lp_assignments)
            branch_var = self.get_branch_var_lp(assignments, diff_dis, lp_assignments)
            # print("lp_assignemnts: ", lp_assignments)
            # print("lp_assignemnts num 0s: ", len([x for x in lp_assignments if x == 0]))
            # print("value of thing we're assigning:", lp_assignments[branch_var - 1], "num assignments:", len([x for x in assignments if x != -1]), ", branch_var: ", branch_var)

            # print("parent_obj_val:", parent_obj_val)
            if self.incumbent_cost < parent_obj_val:
                # print(f"paren val {parent_obj_val} is worse than incumbent {self.incumbent_cost}")
                self.stats.heap_solves_skipped += 1
                continue 
            
            self.search(branch_var, assignments[:], diff_dis.copy())
            self.search(-branch_var, assignments, diff_dis)
            
        self.stats.print()
        return self.incumbent_cost

    def search(self, branch_var: int, fixed: list[int], diff_dis: np.ndarray):
        fixed[abs(branch_var) - 1] = 1 if branch_var > 0 else 0
        feasible, objective_value, lp_assignments = self.lp_solver.solve(fixed)
        self.stats.lp_solves += 1

        if branch_var > 0:
            diff_dis = diff_dis | self.table[abs(branch_var) - 1]
        
        if not feasible:
            self.stats.leaves_infeasible += 1
            pass
        elif objective_value > self.incumbent_cost:
            self.stats.leaves_pruned += 1
            pass 
        elif is_integral_assignments(lp_assignments):
            # print("integral lp_assignments:", lp_assignments)
            self.stats.leaves_integral += 1
            if objective_value < self.incumbent_cost:
                # print(f"{colors.OKBLUE}New incumbent: {objective_value}{colors.ENDC}")
                self.stats.leaves_better_integral += 1
                self.incumbent_cost = objective_value
                self.incumbent_assignment = lp_assignments
        else:
            heapq.heappush(self.heap, (objective_value, fixed, diff_dis, lp_assignments))
    

    def get_branch_var_sequential(self, assignments: list[int]) -> int:
        for i, val in enumerate(assignments):
            if val == -1:
                return i + 1
        return 0

    def get_branch_var_diff_dis(self, assignments: list[int], diff_dis: np.ndarray) -> int:
        # [1, 0, -1]
        unassigned = [i for i, val in enumerate(assignments) if val == -1]
        if not unassigned: return 0

        nondiff_dis = 1 - diff_dis

        unassigned_rows = self.table[unassigned]
        new_diffs = np.bitwise_and(unassigned_rows, nondiff_dis)

        new_diff_counts = np.sum(new_diffs, axis=1)

        # TODO: Change heuristic to not be super greedy on cost
        new_diff_counts = new_diff_counts / self.cost[unassigned]

        best_i = np.argmax(new_diff_counts)
        best_var = unassigned[best_i]

        return best_var + 1

    def get_branch_var_fractional(self, assignments: list[int], diff_dis: np.ndarray, lp_assignments: list[float]) -> int:
        # [1, 0, -1]
        unassigned_and_fractional = [i for i, (is_assigned, value) in enumerate(zip(assignments, lp_assignments)) if is_assigned == -1 and is_not_integer(value)]
        nondiff_dis = 1 - diff_dis

        unassigned_rows = self.table[unassigned_and_fractional]
        new_diffs = np.bitwise_and(unassigned_rows, nondiff_dis)

        new_diff_counts = np.sum(new_diffs, axis=1)

        new_diff_counts = new_diff_counts

        best_i = np.argmax(new_diff_counts)
        best_var = unassigned_and_fractional[best_i]

        return best_var + 1

    def get_branch_var_lp(self, assignments: list[int], diff_dis: np.ndarray, lp_assignments: list[float]) -> int:
        # [1, 0, -1]
        closest_to_one = -1
        distance_to_one = 1
        for i, val in enumerate(lp_assignments):
            if assignments[i] != -1:
                continue
            if 1 - val <= distance_to_one:
                distance_to_one = 1 - val
                closest_to_one = i
        # print(closest_to_one)

        return closest_to_one + 1