import benda
from benda import bjit, u24


@bjit
def simple() -> u24:
    x = u24(3)
    y = x - u24(2)
    return y


val = simple()
print(val)