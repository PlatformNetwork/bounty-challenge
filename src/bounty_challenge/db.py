"""Challenge database exports."""

from __future__ import annotations

from .config import settings
from .sdk.db import Base, Database

database = Database(settings.database_url)

__all__ = ["Base", "database"]
