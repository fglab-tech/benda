import random

import benda
from benda import U24 as u24
book = benda.load_book_from_file("./examples/bitonic_sort.bend")

Sum = book.defs.sum
Sort = book.defs.sort
Gen = book.defs.gen

def main():

    result =  Sum(4, Sort(4, 0, Gen(4)))
    print("Result:  ", result)

if __name__ == "__main__":
    main()
