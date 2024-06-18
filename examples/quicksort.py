from typing import TypeVar
import random

from .quicksort_mock import u24, List, List_Cons, List_Nil, mock_sort

T = TypeVar("T")


# import benda
# from benda import u24
# book = benda.load_book_from_file("./quicksort.bend")
# List_Nil = book.adts.List.Nil
# List_Cons = book.adts.List.Cons


def gen_list(n: int, max_value: int = 0xffffff) -> list[u24]:
    """Generates an array of random u24 numbers with the `random` package"""
    result: list[u24] = []
    for _ in range(n):
        result.append(u24(random.randint(0, max_value)))
    return result


def to_cons_list(xs: list[int]) -> List[u24]:
    """Converts a Python list to a Bend cons-list"""
    result = List_Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = List_Cons(u24(xs[hi]), result)

    return result


def from_cons_list(xs: List[u24]) -> list[u24]:
    """Converts a Bend cons-list to a Python list"""
    result: list[u24] = []
    while True:
        match xs:
            case List_Nil():
                return result
            case List_Cons(value, tail):
                result.append(value)
                xs = tail


def main():
    data = gen_list(10, 1000)
    print("Data:    ", data)

    expected = sorted(data)
    print("Expected:", expected)

    cons_list = to_cons_list(data)

    # sorted_res = book.defs.Sort(cons_list)
    # sorted_arr = from_cons_list(sorted_res)
    # print("Result:  ", sorted_arr)

    mocked_sorted = mock_sort(cons_list)
    mocked_sorted_arr = from_cons_list(mocked_sorted)
    print("Mocked:  ", mocked_sorted_arr)


if __name__ == "__main__":
    main()
