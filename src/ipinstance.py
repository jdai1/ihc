from dataclasses import dataclass
import numpy as np
from docplex.mp.model import Model


@dataclass(frozen=True)
class IPConfig:
    numTests: int  # number of tests
    numDiseases: int  # number of diseases
    costOfTest: np.ndarray  # [numTests] the cost of each test
    A: np.ndarray  # [numTests][numDiseases] 0/1 matrix if test is positive for disease


#  * File Format
#  * #Tests (i.e., n)
#  * #Diseases (i.e., m)
#  * Cost_1 Cost_2 . . . Cost_n
#  * A(1,1) A(1,2) . . . A(1, m)
#  * A(2,1) A(2,2) . . . A(2, m)
#  * . . . . . . . . . . . . . .
#  * A(n,1) A(n,2) . . . A(n, m)


def data_parse(filename: str):
    try:
        with open(filename, "r") as fl:
            numTests = int(fl.readline().strip())  # n
            numDiseases = int(fl.readline().strip())  # m

            costOfTest = np.array([float(i) for i in fl.readline().strip().split()]) # length numT

            A = np.zeros((numTests, numDiseases))
            for i in range(0, numTests):
                A[i, :] = np.array([int(i) for i in fl.readline().strip().split()]) # numT x numD
            return numTests, numDiseases, costOfTest, A
    except Exception as e:
        print(f"Error reading instance file. File format may be incorrect.{e}")
        exit(1)


class IPInstance:
    def __init__(self, filename: str) -> None:
        numT, numD, cst, A = data_parse(filename)
        self.numTests = numT
        self.numDiseases = numD
        self.costOfTest = cst
        self.A = A
        self.model = Model()  # CPLEX solver
        self.usage = self.model.integer_var_list(self.numTests, 0, 1)
        self.table = self.init_table() # numT x ~numD^2

    def init_table(self) -> np.ndarray:
        table = []
        for i in range(self.numDiseases):
            for j in range(i + 1, self.numDiseases):
                table.append(
                    (self.A[:, i] - self.A[:, j])**2 
                )
        table = np.stack(table, axis=1)
        return table
                    

    def solve(self):
        for i in range(len(self.table[0])):
            self.model.add_constraint(
                self.model.scal_prod(
                    terms=self.usage,
                    coefs=self.table[:, i],
                )
                >= 1
            )

        self.model.minimize(
            self.model.scal_prod(terms=self.usage, coefs=self.costOfTest)
        )

        # add extra constraints for fixed values
        temp = []
        self.model.add(temp)
        for c_temp in temp:
            self.model.remove(c_temp)


        sol = self.model.solve()
        cost_celing = self.model.objective_value
        if sol:
            
            for i in range(self.numTests):
                print(self.usage[i].solution_value, end=" ")
            print(cost_celing)

            return
            
        raise Exception("no solution found")
        

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
