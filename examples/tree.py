from dataclasses import dataclass
# from benda import bjit, u24

u24 = int


@dataclass
class Leaf:
    value: u24


@dataclass
class Node:
    left: 'Tree'
    right: 'Tree'


Tree = Node | Leaf


# @bjit
def sum_tree(tree: Tree) -> u24:
    match tree:
        case Leaf(value):
            return value
        case Node(left, right):
            return sum_tree(left) + sum_tree(right)


def gen_tree(depth: int, n: int) -> Tree:
    if depth == 0:
        return Leaf(value=n)
    else:
        return Node(left=gen_tree(depth-1, n-1), right=gen_tree(depth-1, n+1))


if __name__ == "__main__":
    tree = gen_tree(4, 10)
    print(tree)
    print(sum_tree(tree))
