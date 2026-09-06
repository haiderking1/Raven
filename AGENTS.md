# Agent rules

## Keep implementation modular

Give each file one clear responsibility. Group related features in nested directories with focused modules. Never create a god file that owns multiple responsibilities, even if it delegates some work to smaller modules.

## Implement the actual behavior

Do not simplify requirements, take shortcuts, or use hacks to make code compile or pass checks. Fix the underlying problem rather than hiding it. If a requirement is unclear or blocked, ask instead of substituting a reduced implementation.

## Keep tests small and meaningful

Write the smallest set of focused tests that verifies real behavior and catches regressions. Do not add smoke tests or tests that only prove code runs. Avoid redundant cases: do not write ten tests for the same symptom. Add another test only when it covers a distinct behavior or failure mode.
