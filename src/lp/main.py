import json 
import sys 
from pathlib import Path
from model_timer import Timer
from ipinstance import IPInstance

def main(filepath : str):
	
	filename = Path(filepath).name
	watch =  Timer()
	watch.start()
	solver = IPInstance(filepath)
	solver.solve()
	inst_str = solver.toString()
	watch.stop()
	print(inst_str)

	# sol_dict ={
	# 	"Instance" : filename,
	# 	"Time" : str(watch.getElapsed()),
	# 	"Result" : "--",
	# 	"Solution" : "--"
	# }
	# print(json.dumps(sol_dict))	

if __name__ == "__main__":
	if len(sys.argv) != 2:
		print("Usage: python main.py <input_file>")
	main(sys.argv[1])