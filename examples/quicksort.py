import benda
import random
book = benda.load_book_from_file("./examples/quicksort.bend")
List = book.adts.List

def gen_list(n: int, max_value: int = 0xffffff) -> list[int]:
    """Generates an array of random u24 numbers with the `random` package"""
    result: list[int] = []
    for _ in range(n):
        result.append(int(random.randint(0, max_value)))
    return result


def to_cons_list(xs: list[int]):
    """Converts a Python list to a Bend cons-list"""
    result = List.Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = List.Cons(int(xs[hi]), result)

    return result

def from_cons_list(xs) -> list[int]:
    """Converts a Bend cons-list to a Python list"""
    result: list[int] = []
    while True:
        match xs:
            case List.Nil.type():
                return result
            case List.Cons.type(value, tail):
                result.append(value)
                xs = tail

def print_list(list):
    print("[", end="")
    while True:
        match list:
            case book.adts.List.Cons.type(value, tail):
                print(value, end=", ")
                list = tail
            case book.adts.List.Nil.type():
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
