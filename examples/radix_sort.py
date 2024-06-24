import random

import benda
from benda import U24 as u24
book = benda.load_book_from_file("./examples/radix_sort.bend")
Arr = book.adts.Arr

def gen(n, x):
    if n == 0:
        return Arr.Leaf(x)
    else:
        a = x * 2
        b = x * 2 + 1
        return Arr.Node(gen(n - 1, a), gen(n - 1, b))


def main():
    data = gen(10, 0)

    reversed = book.defs.reverse(data)
    sorted_res = book.defs.sort(reversed)
    sum = book.defs.sum(sorted_res)
    print("Result:  ", sum)

if __name__ == "__main__":
    main()

