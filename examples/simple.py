import benda
from benda import bjit, u24

def simple(a) -> u24:
    x = u24(a)
    y = x - u24(2)
    return y


translated_simple = bjit(simple)
val = simple(5)
print(val)

val_bend = translated_simple(5)
print(val_bend)