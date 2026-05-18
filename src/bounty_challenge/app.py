"""FastAPI application entrypoint for Bounty Challenge."""

from __future__ import annotations

from . import models as _models  # noqa: F401 - register SQLAlchemy models
from .config import settings
from .db import database
from .routes import router
from .sdk.app_factory import create_challenge_app
from .weights import get_weights

app = create_challenge_app(
    settings=settings,
    database=database,
    public_router=router,
    get_weights_fn=get_weights,
)

__all__ = ["app"]
