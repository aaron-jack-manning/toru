# Data Invariants

This file contains notes on invariants which must be upheld across running toru commands for the vault and configuration.

### tasks::Task
- name cannot be purely numeric
- ID must be unique

### tasks::Duration
- minutes should be less than 60

### state::State (in state.toml)
- the `next_id` should always be greater than the ID of any task within the vault

### index::Index (in state.toml)
- the index of name to ID map should be correct given the names and IDs of all vaults

### graph::Graph (in state.toml)
- should always have dependencies the same as specified in each task file
- no circular dependencies should be allowed to exist between tasks (and by extension should not exist in the state file)
