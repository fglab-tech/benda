
import benda
book = benda.load_book_from_file("./examples/sat_solver.bend")

pred = book.defs.pred

def main():
    x1 = [0,1]
    x2 = [0,1]

    for i in x1:
        for x in x2:
            result =  pred(i, x)
            print("Result:  ", result)


if __name__ == "__main__":
    main()
