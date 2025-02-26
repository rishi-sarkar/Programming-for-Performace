# Concurrent HashMap Implementation

## Pull Request Title
Introduce concurrent HashMap to improve parallel performance

## Summary
This update replaced standard hash maps with DashMap to allow safe concurrent modifications across threads, significantly improving performance in multi-threaded scenarios. Previously, the implementation relied on a standard HashMap wrapped in a mutex, which caused bottlenecks in high-concurrency environments. This change enhanced scalability, particularly for workloads that involve extensive dictionary updates, and enabled more efficient usage of system resources in multi-threaded executions.

## Technical Details
One of the main challenges with the previous implementation was synchronization overhead due to locking mechanisms around HashMap. Since multiple threads were frequently accessing and updating token counts, contention became a significant performance bottleneck, reducing the benefits of multi-threading. By introducing DashMap, which provides built-in thread safety and lock-free operations for individual entries, these issues were mitigated to achieve better concurrency.

The process_dictionary_builder_line function was refactored to take advantage of DashMap's concurrent insertions. This allowed multiple threads to simultaneously process different parts of the input data without blocking each other, leading to substantial performance improvements.

Another critical optimization was the efficient merging of results at the end of processing. This was resolved by utilizing atomic operations to restructure the final aggregation phase into working efficiently within a multi-threaded execution.

## Testing for Correctness
- Conducted functional tests to ensure token extraction results remain identical.
- Verified that no race conditions occur when running with multiple threads.

## Performance Testing
- Compared execution time with single-threaded and multi-threaded approaches.
- Observed significant speedup for large log files, demonstrating better thread utilization.

