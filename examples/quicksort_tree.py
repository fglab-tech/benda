import benda
import random

book = benda.load_book_from_file("./examples/quicksort_tree.bend")

def gen_list(n: int, max_value: int = 0xffffff):
    """Generates an array of random u24 numbers with the `random` package"""
    result = []
    for _ in range(n):
        result.append(random.randint(0, max_value))
    return result


def to_cons_list(xs: list[int]):
    """Converts a Python list to a Bend cons-list"""
    result = book.adts.List.Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = book.adts.List.Cons(xs[hi], result)

    return result

def print_tree(tree):
    match tree:
        case book.adts.MyTree.Node.type(left, value,right):
            print_tree(left)
            print(value, end= ", ")
            print_tree(right)
        case book.adts.MyTree.Leaf.type():
            return


if __name__ == "__main__":

    pylist = gen_list(50,5000)

    tree = to_cons_list(pylist)
    sort_tree = book.defs.Sort(tree)

    pylist.sort()

    print("Python Sort: ")
    print(pylist)

    print("Bend Tree Quicksort: ")
    print_tree(sort_tree.to_adt(book.adts.MyTree))
    print()
