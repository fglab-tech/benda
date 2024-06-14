from typing import TypeVar, Generic
from dataclasses import dataclass
import random

T = TypeVar("T")

# --> Our Target

# import benda
# from benda import u24

# book = benda.load_book_from_file("./quicksort.bend")
# List_Nil = book.adts.List.Nil
# List_Cons = book.adts.List.Cons
# MyTree_Leaf = book.adts.MyTree.Leaf
# MyTree_Node = book.adts.MyTree.Node


# -> Mock

u24 = int


@dataclass
class List_Nil:
    pass


@dataclass
class List_Cons(Generic[T]):
    value: T
    tail: "List[T]"


List = List_Nil | List_Cons[T]


@dataclass
class MyTree_Leaf:
    pass


@dataclass
class MyTree_Node(Generic[T]):
    value: T
    left: "MyTree[T]"
    right: "MyTree[T]"


MyTree = MyTree_Leaf | MyTree_Node[T]


def gen_list(n: int, max_value: int) -> List[u24]:
    if n <= 0:
        return List_Nil()
    else:
        value = random.randint(0, max_value)
        return List_Cons(u24(value), gen_list(n-1, max_value))


def print_tree(tree: MyTree[u24]):
    match tree:
        case MyTree_Leaf():
            pass
        case MyTree_Node(value, left, right):
            print(value)
            print_tree(left)
            print_tree(right)


def list_to_tree(list: List[u24]) -> MyTree[u24]:
    match list:
        case List_Nil():
            return MyTree_Leaf()
        case List_Cons(value, tail):
            return MyTree_Node(value, MyTree_Leaf(), list_to_tree(tail))


def mock_sum(tree: MyTree[u24]) -> u24:
    match tree:
        case MyTree_Leaf():
            return u24(0)
        case MyTree_Node(value, left, right):
            return value + mock_sum(left) + mock_sum(right)


def main():
    numbers = gen_list(7, 1000)
    print(numbers)

    # tree = benda.run(book.defs.Sort, [numbers])
    # tree = book.defs.Sort(numbers)

    tree = list_to_tree(numbers)
    print("Values:")
    print_tree(tree)

    # result = book.run(book.defs.Sum, [tree])

    result = mock_sum(tree)
    print("Result:", result)


if __name__ == "__main__":
    main()
