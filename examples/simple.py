import ast, inspect

import benda
from benda import u24

def test_1():
    return 4

def test_2():
    x = u24(3)
    y = x - u24(2)
    return y

def get_ast():
    my_ast = ast.dump(ast.parse(inspect.getsource(test_2)))
    return my_ast

if __name__ == "__main__":
    print(get_ast())
    print(test_2())
