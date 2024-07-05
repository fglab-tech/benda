import benda

book = benda.load_book_from_file("./examples/dpll.bend")
List = book.adts.List

def generate_clause(v1, v2):
    return List.Cons(v1, List.Cons(v2, List.Nil()))

def generate_formula():
    fir = generate_clause(1, 2)
    #fir = List.Cons(2, List.Nil())
    sec = generate_clause(-1, 3)
    thi = generate_clause(2, 3)
    fou = generate_clause(-3, 2)

    return List.Cons(fir, List.Cons(sec, List.Cons(thi, List.Cons(fou, List.Nil()))))

def generate_vars():
    return List.Cons(1, List.Cons(2, List.Cons(3, List.Nil())))

def print_list(my_list):
    while True:
        match my_list:
            case List.tCons(value, tail):
                print(value)
                my_list = tail
            case List.tNil():
                break

prop = book.defs.simplify_formula(generate_formula(), 1, 3, -1, 1).to_adt(List)

#print_list(prop)

#prop = book.defs.unit_propagation(generate_formula(), 1, generate_vars())
#
#print(prop)
#
while True:
    match prop:
        case List.tCons(value, tail):
            while True:
                match value:
                    case List.tCons(val, tail_val):
                        print(val)
                        value = tail_val
                    case List.tNil():
                        print()
                        break
            prop = tail
        case List.tNil():
            break
