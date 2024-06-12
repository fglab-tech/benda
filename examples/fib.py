from typing import TypeVar
T = TypeVar("T")


def bend(fn: T) -> T:
    """
    Decorator to hind compiler that the recursive function should be translated
    to a inline `bend` block.

    This will be internal to the library.
    """
    return fn


def fib_recursive(n: int) -> int:
    """
    Calculates fibonacci numbers recursively.
    """
    match n:
        case 0:
            return 0
        case 1:
            return 1
        case _:
            return fib_recursive(n - 2) + fib_recursive(n - 1)


def fib_iterative(n: int) -> int:
    """
    Calculates fibonacci numbers iteratively (tail-recursively).
    """

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
