<h1>Bend FFI for Python</h1>
<p>Use Bend functions, data structures and <strong>speed</strong> in Python</p>

## Index
1. [Introduction](#introduction)
2. [Functions](#functions)
3. [Book](#book)
4. [ADTs](#adts)
5. [Definitions](#definitions)

## Introduction

If there is some part of your Python code that needs speed and you have a Bend function that does the job, you can use the Benda FFI to call that function from Python. Use the power of Bend to speed up your Python code.

## Functions

The `benda ffi` has the following functions:

- `load_book_from_file()` - Load a book from a file, returns a book object;

## Book

A book object has the following uses:

- `book.adts` - Get the adts of the book. Example: `book.adts.List`;
- `book.defs` - Get the definitions, or bend functions, of the book. Example: `book.defs.Sort()`;

## ADTs

A ADT is a way to define a data structure in Bend. The Benda FFI can load the ADTs defined in a Book and expose them in Python. All the builtin ADTs of Bend are in any book loaded.<br>
The way to use a ADT is to access it from a `adts` object. Every ADT is composed of a set of `Constructors`, and each of these represent a instance of the ADT.<br>
Example:

``` python
def to_cons_list(xs: list[int]):
    result = book.adts.List.Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = book.adts.List.Cons(xs[hi], result)

    return result

```

In this example, we are creating a `List` ADT from a Python list. The `List` ADT has two constructors: `Nil` and `Cons`. We are using the `Nil` constructor to represent the end of the `List` and the `Cons` constructor to represent an element of the `List`.<br>
These ADTs can be accessed using `match` statements to extract the values from the ADT. Example:

``` python

my_list = to_cons_list([1, 2, 3, 4, 5])

while True:
    match my_list:
        case book.adts.List.tCons(value, tail):
            print(value)
            my_list = tail
        case book.adts.List.tNil():
            print("End")
            break

```

Notice that the Constructors in this case are prefixed with a `t` to indicate that you are matching the `types` of the constructors. Every time you want to pattern match a ADT, you need to prefix the constructor with a `t`.<br>

## Definitions

A definition is a Bend function that can be called from Python. The Benda FFI can load the definitions defined in a Book and expose them in Python. All the builtin definitions of Bend are in any book loaded.<br>
The way to use a definition is to access it from a `book.defs` object. Benda checks the number of arguments the Bend function needs.<br>Example:

``` python
import random
import benda

book = benda.load_book_from_file("./examples/quicksort.bend")

def gen_list(n: int, max_value: int):
    if n <= 0:
        return book.adts.List.Nil()
    else:
        value = random.randint(0, max_value)
        return book.adts.List.Cons(value, gen_list(n-1, max_value))

my_list = gen_list(400, 200)
sorted_list = book.defs.Sort(my_list)

```

In this example, we are generating a list of 400 random numbers and sorting it using the `Sort` definition. The `Sort` definition is a Bend function that takes a `List` and returns a sorted `List` Term.<br>
Every definition call in Benda returns a `Term` object. This object represents the output of [HVM](https://github.com/HigherOrderCO/HVM) in lambda encoding. To convert this object into a ADT, you can use the `to_adt` function.<br>Example:

``` python
sorted_list = book.defs.Sort(my_list).to_adt(book.adts.List)
```

This way you can convert the `Term` object into a ADT to use complex data structures in Python.<br>

## Superpositions

Use [superpositions](https://gist.github.com/VictorTaelin/9061306220929f04e7e6980f23ade615) to boost your code performance. Superpositions are a way to call a Bend function multiple times using a `Tuple` of values.<br>
Example:
```python
import benda
book = benda.load_book_from_file("./examples/sat_solver.bend")

pred = book.defs.pred

x1 = benda.Fan(0,1)
x2 = benda.Fan(0,1)

result =  pred(x1,x2)
print("Result:  ", result)

```

Here, Fan represents a superposition of values. This code applies the `pred` function to all the possible values of x1 and x2. The result is a superposition of the results of the function applied to all the possible values of x1 and x2.<br>
