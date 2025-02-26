import csv
import math

def compare(i, l1, l2):
    assert len(l1) == len(l2)
    for j in range(len(l1)):
        if not math.isclose(l1[j], l2[j]):
            print ("line {} number {}: {} {}".format(i+1, j+1, l1[j],l2[j]))
            exit(1)


out_file = "output/out.csv"
out_cuda_file = "output/out_cuda.csv"

with open(out_file, 'r') as f1:
    reader = csv.reader(f1)
    out_lines = [[float(n) for n in line] for line in reader]

with open(out_cuda_file, 'r') as f2:
    reader = csv.reader(f2)
    out_cuda_lines = [[float(n) for n in line] for line in reader]

assert len(out_lines) == len(out_cuda_lines)
for i in range(len(out_lines)):
    compare(i, out_lines[i], out_cuda_lines[i])

print("Comparison finished")
