<h1>Bend FFI for Python</h1>
<p>Use Bend functions, data structures and <strong>capabilities</strong> in Python</p>

## Index
1. [Introduction](#introduction)
2. [Functions](#functions)
3. [Book](#book)
4. [ADTs](#adts)
5. [Definitions](#definitions)

## Introduction

When you want to use Bend on your Python code, Benda allows you to integrate Bend into Python using its ADTs, functions and capabilities.

## Functions

The `benda ffi` module provides the following key function:

- `load_book_from_file(file_path: str) -> Book`: Loads a Bend book from the specified file path and returns a Book object.

Example usage:
```python
from benda import load_book_from_file

book = load_book_from_file("./path/to/your/bendbook.bend")
```

## Book

A book object has the following uses:

- `book.adts` - Get the adts of the book. Example: `book.adts.List`;
- `book.defs` - Get the definitions, or bend functions, of the book. Example: `book.defs.Sort()`;
<br>
You can modify the Book's runtime environment using the `book.set_cmd()` function. This function accepts an argument of type `BendCommand`, an Enum that specifies the desired runtime. Available options include:

- `BendCommand.Rust`: Use the Rust runtime
- `BendCommand.C`: Use the C runtime
- `BendCommand.Cuda`: Use the CUDA runtime for GPU acceleration

Example usage:
```python
from benda import BendCommand

book.set_cmd(BendCommand.Cuda)  # Set the runtime to Cuda
```

Choose the appropriate runtime based on your performance requirements and available hardware

## ADTs

Abstract Data Types (ADTs) in Bend provide a powerful way to define complex data structures. The Benda FFI seamlessly loads ADTs defined in a Bend Book and makes them accessible in Python. Every loaded Book includes all of Bend's built-in ADTs, ensuring you have access to a rich set of data structures out of the box.<br>
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
List = book.adts.List

while True:
    match my_list:
        case List.Cons.type(value, tail):
            print(value)
            my_list = tail
        case List.Nil.type():
            print("End")
            break

```

Notice that you're matching against the `type` attribute of a Constructor, so everytime you need to use a `match` with an ADT, you need to use this attribute. This is due to [pyo3](https://pyo3.rs/v0.22.1/br) limitation of creating types at runtime.<br>

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

<!-- ## Superpositions

Leverage [superpositions](https://gist.github.com/VictorTaelin/9061306220929f04e7e6980f23ade615) to significantly enhance your code's performance. Superpositions allow you to efficiently apply a Bend function to multiple input values simultaneously, exploiting parallelism and reducing overall computation time.<br>

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

Here, Fan represents a superposition of values. This code applies the `pred` function to all the possible values of x1 and x2. The result is a superposition of the results of the function applied to all the possible values of x1 and x2.<br> -->
