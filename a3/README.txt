This lab uses the CNN algorithm described in the instructions.

The input files consist of a CNN file describing the CNN and an input file containing a number of input matrices.
The output files contain the output vectors for each input matrix.

To run the code, use:

    cargo run --release -- <mode> <cnn_file> <input_file> <output_file>

where <mode> is either "cpu" "cuda". All the files are pathnames (unlike in
the earlier Sudoku example).  You would typically use the following commands:

    cargo run --release -- cpu input/cnn.csv input/in.csv output/out.csv

    cargo run --release -- cuda input/cnn.csv input/in.csv output/out_cuda.csv

The program outputs the time spent doing "actual work", which is the
work of converting input matrices to output vectors.  This measurement
does not include I/O or the overhead of initializing the GPU. As such,
the time should be lower for the CUDA version than the CPU version.

The repo also includes 2 helper scripts, written in Python:

  generate.py generates random CNNs and input matrices, defaulting to
     "input/cnn.csv" and "input/in.csv"

  compare.py compares 2 output matrices to see if they are "close enough".
      Used to test the implementations for correctness.
      Reads output/out.csv and output/out_cuda.csv

You can run each of them using the python3 command (e.g. "python3 compare.py").

