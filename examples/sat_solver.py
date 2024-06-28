
import benda
book = benda.load_book_from_file("./examples/sat_solver.bend")

pred = book.defs.pred

def main():
    x1 = benda.Fan(0,1)
    x2 = benda.Fan(0,1)

    result =  pred(x1,x2)
    print("Result:  ", result)


if __name__ == "__main__":
    main()
