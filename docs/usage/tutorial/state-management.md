# State Management

Ref:

- <https://tauri.app/develop/state-management/>
- [pytauri.State][]
- [pytauri.Manager.state][]
- [pytauri.Manager.manage][]

PyTauri implements state management API consistent with Rust Tauri. Reading Tauri's documentation is like reading PyTauri's documentation.

## Managing and Accessing State

```python
--8<-- "docs_src/tutorial/state_management/managing_accessing.py"
```

## State injection in `Commands`

You can inject state into any Command, with any type and any parameter name, as long as you use `Annotated[T, State()]` as its type annotation.

!!! note
    You must [Manager.manage][pytauri.Manager.manage] these states before you invoke the command, or the invocation will be rejected.

```python
--8<-- "docs_src/tutorial/state_management/state_injection.py"
```
