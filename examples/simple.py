import ast
import inspect

import benda
from benda import u24


def simple():
    x = u24(3)
    y = x - u24(2)
    return y


if __name__ == "__main__":
    translated_simple = bjit(simple)
    print(simple())
    print(translated_simple())
