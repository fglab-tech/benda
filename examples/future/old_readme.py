from dataclasses import dataclass
from benda import bjit, u24

@dataclass
class Leaf:
  value: u24  # native HVM machine integer

@dataclass
class Node:
  left: 'Tree'
  right: 'Tree'

Tree = Node | Leaf

# The `bjit` decorator will introspect and translate the function to HVM/Bend
# code, replacing it with a wrapper that converts the Python-level types of the
# inputs and result value, Numba-style.

@bjit
def sum_tree(tree: Tree) -> u24:
  match tree:
    case Leaf(value=value):
      return value
    case Node(left=left, right=right):
      return sum_tree(left) + sum_tree(right)
    case _:
      raise TypeError("Invalid type for tree")

# Alternatively, you can opt to use Python big integers and other primitives,
# they will be translated to the equivalent representations automatically.

@dataclass
class Leaf2:
  value: int
