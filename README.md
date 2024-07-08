# Benda

[Numba]-like Python translation to [HVM]/[Bend].

[Numba]: https://numba.pydata.org/
[HVM]: https://github.com/HigherOrderCo/hvm
[Bend]: https://github.com/HigherOrderCo/bend

Read the Docs:<br>
[FFI](/docs/FFI.md)

This is in MVP stage.

## Example

```py
import benda
import random

book = benda.load_book("""
(Sort List/Nil) = List/Nil
(Sort(List/Cons head tail)) =
((Part head tail) λmin λmax
 let lft=(Sort min)
 let rgt=(Sort max)
 (Concat lft(List/Cons head rgt)))

# Partitions a list in two halves, less-than-p and greater-than-p
(Part p List/Nil) = λt(t List/Nil List/Nil)
(Part p(List/Cons head tail)) = (Push(> head p) head(Part p tail))

# Pushes a value to the first or second list of a pair
(Push 0 x pair) = (pair λmin λmax λp(p(List/Cons x min) max))
(Push _ x pair) = (pair λmin λmax λp(p min(List/Cons x max)))

(Concat List/Nil tail) = tail
(Concat(List/Cons head tail) xs2) =
(List/Cons head(Concat tail xs2))
""")

List = book.adts.List

def gen_list(n: int, max_value: int = 0xffffff) -> list[int]:
    result: list[int] = []
    for _ in range(n):
        result.append(random.randint(0, max_value))
    return result


def to_cons_list(xs: list[int]):
    result = List.Nil()

    hi = len(xs)
    if hi == 0:
        return result

    while hi > 0:
        hi -= 1
        result = List.Cons(xs[hi], result)

    return result

def print_cons_list(list):
    while True:
        match list:
            case List.Cons.type(value, tail):
                print(value, end=", ")
                list = tail
            case List.Nil.type():
                break


data = gen_list(5, 1000)
cons_list = to_cons_list(data)
book.set_cmd(benda.BendCommand.Cuda)
sorted_list = book.defs.Sort(cons_list)
sorted_list = sorted_list.to_adt(book.adts.List)
print_cons_list(sorted_list)
```

## Install

To install the current release:
```
$ pip install benda
```

## Development

Dependencies:

- Python 3.11+
- Rust
- C compiler
- maturin

### Getting dependencies with Nix (optional)

- Install Nix with [Determinate Nix Installer]

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf -L \
    https://install.determinate.systems/nix | sh -s -- install
  ```

- You can run `nix develop` to enter a shell with the dependencies installed.

### Building

- Create and activate a Python virtual environment.
  - e.g. with
    ```
    python -m venv .venv
    source .venv/bin/activate
    ```

- Run `make` to build the project and install the `benda` package in the virtual
  environment.

<!-- - You can use [`direnv`][direnv] to automatically load the environment when you
  enter the project directory. -->

[Determinate Nix Installer]: https://install.determinate.systems
[direnv]: https://direnv.net
