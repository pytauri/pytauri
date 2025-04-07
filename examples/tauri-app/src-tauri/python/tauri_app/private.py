"""This is an example to demonstrate how to use Cython to protect your source code in a pytauri standalone app."""


def private_algorithm(data: int) -> int:
    """A private algorithm that you don't want users to know."""

    # this is a fake algorithm, just for demonstration
    output = data * 2

    return output
