"""Challenge weight computation from owner-set emissions."""

from __future__ import annotations

import math

from sqlalchemy import select

from .db import database
from .models import Emission


async def get_weights() -> dict[str, float]:
    """Return raw owner-set hotkey weights for the Platform master to normalize."""

    statement = select(Emission).where(Emission.weight > 0).order_by(Emission.weight.desc())
    async with database.session() as session:
        rows = (await session.execute(statement)).scalars().all()

    weights: dict[str, float] = {}
    for row in rows:
        weight = float(row.weight)
        if math.isfinite(weight) and weight > 0:
            weights[row.hotkey] = weight
    return weights
