from typing import TypeVar
import random

from .quicksort_mock import u24, List_Cons, List_Nil, mock_sort
from .quicksort_mock import List as ListMock

T = TypeVar("T")


import benda
from benda import U24
book = benda.load_book_from_file("./examples/quicksort.bend")
List = book.adts.List
# List_Nil = book.adts.List.Nil
# List_Cons = book.adts.List.Cons


def gen_list(n: int, max_value: int = 0xffffff) -> list[u24]:
    """Generates an array of random u24 numbers with the `random` package"""
    result: list[u24] = []
    for _ in range(n):
        result.append(u24(random.randint(0, max_value)))
    return result


def to_cons_list(xs: list[int]):
    """Converts a Python list to a Bend cons-list"""
    result = List.Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = List.Cons(u24(xs[hi]), result)

    return result

# Ideal Syntax:
#def from_cons_list(xs: List[u24]) -> list[u24]:
#    """Converts a Bend cons-list to a Python list"""
#    result: list[u24] = []
#    while True:
#        match xs:
#            case List_Nil():
#                return result
#            case List_Cons(value, tail):
#                result.append(value)
#                xs = tail

def from_cons_list(xs) -> list[u24]:
    """Converts a Bend cons-list to a Python list"""
    result: list[u24] = []
    while True:
        match xs:
            case List.tNil():
                return result
            case List.tCons(value, tail):
                result.append(value)
                xs = tail

def print_list(list):
    print("[", end="")
    while True:
        match list:
            case book.adts.List.tCons(value, tail):
                print(value, end=", ")
                list = tail
            case book.adts.List.tNil():
                break
    print("]")


def main():
    data = gen_list(5, 1000)
    print("Data:    ", data)

    expected = sorted(data)
    print("Expected:", expected)

    cons_list = to_cons_list(data)

    sorted_res = book.defs.Sort(cons_list)
    sorted_arr = sorted_res.to_adt(book.adts.List)

    sum = book.defs.Sum(sorted_res)


    print("Result:   ", end="")
    print_list(sorted_arr)
    print("Sum: ", sum)

    #mocked_sorted = mock_sort(cons_list)
    #mocked_sorted_arr = mocked_from_cons_list(mocked_sorted)
    #print("Mocked:  ", mocked_sorted_arr)


if __name__ == "__main__":
    main()
