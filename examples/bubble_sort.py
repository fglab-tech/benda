import random

import benda
from benda import U24 as u24
book = benda.load_book_from_file("./examples/bubble_sort.bend")

Rnd = book.defs.rnd
Sort = book.defs.sort
Sum = book.defs.sum

def main():

    result =  Sum(Sort(Rnd(100)))
    print("Result:  ", result)

if __name__ == "__main__":
    main()




