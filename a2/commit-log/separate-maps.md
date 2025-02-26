# Separate Maps Implementation

## Pull Request Title
Implement separate maps for double and triple token tracking

## Summary
This update introduces separate maps for storing double and triple token occurrences, improving on potential optimizations in future processing. Previously, both double and triple token frequencies were stored in a single data structure, which led to unnecessary complexity and potential inefficiencies when accessing specific data. By introducing separate maps, the code becomes easier to maintain and optimize in the future.

## Technical Details
The key modification in this update is the restructuring of the data storage mechanism. Instead of using a single hash map to store both double and triple token frequencies, two separate hash maps are now used. This allows for improved data organization and optimized retrieval times when performing lookups. The dictionary_builder function has also been updated to manage the separate maps correctly. Additionally, modifications were made to the parsing logic to ensure that tokens are categorized correctly into the new structure without introducing any inconsistencies.

Beyond the basic restructuring, helper functions were added to operate efficiently within this new architecture, ensuring that future changes can be added upon this improved structure without requiring additional significant changes. This design change was carefully integrated with existing functions to maintain compatibility and to ensure that existing test cases pass without any issues.

## Testing for Correctness
- Verified that output remains consistent with the previous implementation.
- Ran unit tests, ensuring expected results for token processing and storage.

## Performance Testing
- Measured execution time before and after the change; observed slight improvements due to reduced contention.
- Verified that separate maps do not introduce performance regressions.