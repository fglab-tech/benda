from dataclasses import dataclass
from benda import bjit, u24

@dataclass
class MyLeaf:
    value: u24


@dataclass
class MyNode:
    left: 'MyTree'
    right: 'MyTree'


MyTree = MyNode | MyLeaf


@bjit
def sum_tree(tree: MyTree) -> u24:
    match tree:
        case MyLeaf(value):
            return value
        case MyNode(left, right):
            return sum_tree(left) + sum_tree(right)


def gen_tree(depth: int, n: int) -> MyTree:
    if depth == 0:
        return MyLeaf(value=n)
    else:
        return MyNode(left=gen_tree(depth-1, n-1), right=gen_tree(depth-1, n+1))


if __name__ == "__main__":
    tree = gen_tree(4, 10)
    #print(tree)
    print(sum_tree(tree))

