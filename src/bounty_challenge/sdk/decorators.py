"""Route decorators used by Platform proxy discovery."""

from __future__ import annotations

from collections.abc import Callable
from typing import TypeVar

F = TypeVar("F", bound=Callable)


def public_route(**metadata: object):
    def decorator(func: F) -> F:
        func.__platform_public_route__ = True
        func.__platform_route_metadata__ = metadata
        return func

    return decorator
