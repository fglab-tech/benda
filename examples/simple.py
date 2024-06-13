import benda
from benda import bjit, u24


def simple() -> u24:
    x = u24(3)
    y = x - u24(2)
    return y


translated_simple = bjit(simple)
print(simple())
print(translated_simple())