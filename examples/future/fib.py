from typing import TypeVar
from benda import bjit, bend

T = TypeVar("T")


@bjit
def fib_recursive(n: int) -> int:
    """
    Calculates the nth Fibonacci number using a recursive approach.
    """
    match n:
        case 0:
            return 0
        case 1:
            return 1
        case _:
            return fib_recursive(n - 2) + fib_recursive(n - 1)


@bjit
def fib_iterative(n: int) -> int:
    """
    Calculates the nth Fibonacci number using an iterative (tail-recursive) approach.
    """

    # This decorator hints the Benda transpiler to use a `bend` block for the
    # recursive computation.
    @bend
    def go(a: int, b: int, n: int) -> int:
        if n == 0:
            return a
        else:
            return go(b, a + b, n - 1)

    return go(0, 1, n)


def main():
    print(fib_iterative(10))
    print(fib_recursive(10))


if __name__ == "__main__":
    main()
