from typing import TypeVar, Generic
from dataclasses import dataclass

T = TypeVar("T")


u24 = int

# Explain


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


def partition(list: List[u24], p: u24) -> tuple[List[u24], List[u24]]:
    left = List_Nil()
    right = List_Nil()

    while True:
        match list:
            case List_Nil():
                return left, right
            case List_Cons(value, tail):
                if value <= p:
                    left = List_Cons(value, left)
                else:
                    right = List_Cons(value, right)
                list = tail


def concat(xs1: List[u24], xs2: List[u24]) -> List[u24]:
    while True:
        match xs1:
            case List_Nil():
                return xs2
            case List_Cons(value, tail):
                return List_Cons(value, concat(tail, xs2))


def mock_sort(list: List[u24]) -> List[u24]:
    match list:
        case List_Nil():
            return List_Nil()
        case List_Cons(value, tail):
            [left, right] = partition(tail, value)
            left = mock_sort(left)
            right = mock_sort(right)
            return concat(left, List_Cons(value, right))
