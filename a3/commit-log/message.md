# Implement GPU-Accelerated CNN with CUDA Kernels

## Summary
This commit introduces significant enhancements by refactoring our CNN implementation to leverage GPU acceleration. Convolution, ReLU activation, and output layer processing are moved from the CPU to the GPU, allowing for parallel execution and potentially improved performance.

## Technical Details

1. **Kernel Workflow**  
   The CUDA kernels (in `kernel.cu`) are separated into three stages:
   - **Convolution Layer:** Computes a 20×20 output per filter by sliding a 5×5 kernel across the input image.
   - **ReLU Layer:** Applies the ReLU activation, zeroing out negative values.
   - **Output Layer:** Uses a parallel reduction strategy to calculate the dot product between the flattened output and each neuron’s weights.

2. **CUDA Context Initialization**  
   In `cuda.rs`, the **init** method sets up the GPU environment. It initializes CUDA via and creates a context with create_and_push, and loads the pre-compiled PTX module. It also allocates memory for CNN layers (e.g., conv_layer, output_layer) on the device.

3. **Launching Kernels**  
   The **compute** method prepares device buffers for the input and output, configuring grid and block dimensions to match the required parallelism. Each kernel is launched in sequence on a single CUDA stream, with a single synchronize() call at the end to ensure proper ordering and data integrity.

4. **Memory Management & Safety**  
   I used on DeviceBox and DeviceBuffer for memory allocations, to manage and transfer data to/from the GPU while minimizing risk. By encapsulating unsafe operations within these abstractions, I reduced the chances of error.

## Correctness Testing
I verified the correctness of the implementation using compare.py, which is designed to output "Comparison Finished" if the comparison was deemed correct showing that the implementation was right.

## Performance Evaluation
Performance was evaluated using the number of microseconds the work took to finish and comparing the results between the two methods.
