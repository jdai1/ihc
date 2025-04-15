from dataclasses import dataclass
import numpy as np
from docplex.mp.model import Model


#  * File Format
#  * #Tests (i.e., n)
#  * #Diseases (i.e., m)
#  * Cost_1 Cost_2 . . . Cost_n
#  * A(1,1) A(1,2) . . . A(1, m)
#  * A(2,1) A(2,2) . . . A(2, m)
#  * . . . . . . . . . . . . . .
#  * A(n,1) A(n,2) . . . A(n, m)


class LPSolver:
    def __init__(self, filename: str) -> None:
        numT, numD, cst, A = self.data_parse(filename)
        self.numTests = numT
        self.numDiseases = numD
        self.costOfTest = cst
        self.A = A

        # pre-processing
        table = []
        for i in range(self.numDiseases):
            for j in range(i + 1, self.numDiseases):
                table.append((self.A[:, i] - self.A[:, j]) ** 2)
        table = np.stack(table, axis=1)

        # establishing model & constraints
        self.model = Model()
        # self.model.context.cplex_parameters.threads = 1

        # decision variables
        self.usage = self.model.continuous_var_list(self.numTests, 0, 1)

        for i in range(len(table[0])):
            self.model.add_constraint(
                self.model.scal_prod(
                    terms=self.usage,
                    coefs=table[:, i],
                )
                >= 1
            )
        self.model.minimize(
            self.model.scal_prod(terms=self.usage, coefs=self.costOfTest)
        )

    def solve(self, assignments: list[int]):
        # add extra constraints for fixed values
        fixed_value_constraints = []
        for usage_var in range(len(assignments)):
            if assignments[usage_var] == -1:
                continue

            fixed_value_constraints.append(
                self.model.add_constraint(
                    self.usage[usage_var] == assignments[usage_var]
                )
            )

        sol = self.model.solve()
        self.model.remove_constraints(fixed_value_constraints)
        # for constr in fixed_value_constraints:
            # self.model.remove_constraint(constr)
        if sol:
            return (
                True,
                self.model.objective_value,
                [self.usage[i].solution_value for i in range(self.numTests)],
            )
        return (False, 0, [])

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

    def toString(self):
        out = ""
        out = f"Number of test: {self.numTests}\n"
        out += f"Number of diseases: {self.numDiseases}\n"
        cst_str = " ".join([str(i) for i in self.costOfTest])
        out += f"Cost of tests: {cst_str}\n"
        A_str = "\n".join(
            [" ".join([str(j) for j in self.A[i]]) for i in range(0, self.A.shape[0])]
        )
        out += f"A:\n{A_str}"
        return out
