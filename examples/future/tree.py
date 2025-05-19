from dataclasses import dataclass
import random

from benda import bjit, u24


@dataclass
class MyLeaf:
    value: u24


@dataclass
class MyNode:
    left: 'MyTree'
    right: 'MyTree'


MyTree = MyNode | MyLeaf


def sum_tree(tree: MyTree) -> u24:
    """
    Recursively sum the values of the tree.
    """
    match tree:
        case MyLeaf(value):
            return value
        case MyNode(left, right):
            return sum_tree(left) + sum_tree(right)


def gen_tree(depth: int) -> MyTree:
    n = random.randint(0, 1000)
    if depth == 0:
        return MyLeaf(value=n)
    else:
        return MyNode(left=gen_tree(depth-1), right=gen_tree(depth-1))


if __name__ == "__main__":
    tree = gen_tree(4)
    print(tree)
    print(sum_tree(tree))
