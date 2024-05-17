# Benda

Numba-like Python translation to HVM.

This is in conceptual stage.

## Example

```py
from dataclasses import dataclass
from benda import bend, u24

@dataclass
class Leaf:
  value: u24  # native HVM machine integer

@dataclass
class Node:
  left: Tree
  right: Tree

Tree = Node | Leaf

# The `bend` decorator will introspect and translate the function to HVM/Bend
# code, replacing it with a wrapper that converts the Python-level types of the
# inputs and result value, Numba-style.

@bend
def sum_tree(tree: Tree) -> u24:
  match tree:
    Leaf(value):
      return value
    Node(left, right):
      return sum_tree(left) + sum_tree(right)


# You can opt to use Python big integers and other primitives just fine, they
# will be translated to the equivalent representations automatically by the lib.

@dataclass
class PyLeaf:
  value: int
```
