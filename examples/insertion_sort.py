import random
import benda

book = benda.load_book_from_file("./examples/insertion_sort.bend")
List = book.adts.List


def rnd(n):
  if n == 0:
    return List.Nil()
  else:
    return List.Cons(random(10 - n), rnd(n - 1))


def random(n):
  if n == 0:
    return 0
  else:
    return (random(n - 1) * 16 + 101387) % 429453


def main():
    data = rnd(10)

    result = book.defs.insertion_sort(data)
    print("Result:  ", result.to_list())


if __name__ == "__main__":
    main()
