from random import uniform
from sys import argv

MAX = 10
inputname = "in.csv"
cnnname = "cnn.csv"

if len(argv) > 1: inputname = argv[1]
if len(argv) > 2: cnnname =  argv[2]

with open("input/"+inputname, 'w') as f:
    num = 250
    f.write(str(num)+'\n')
    for x in range(num):
        for i in range(100):
            row = [str(uniform(-MAX, MAX)) for n in range(100)]
            row = ','.join(row)+'\n'
            f.write(row)
        f.write('\n')

with open("input/"+cnnname, 'w') as f:
    for i in range(10):
        row = [str(uniform(-MAX, MAX)) for n in range(25)]
        row = ','.join(row)+'\n'
        f.write(row)

    f.write('\n')
    for i in range(10):
        row = [str(uniform(-MAX, MAX)) for n in range(4000)]
        row = ','.join(row)+'\n'
        f.write(row)
