"""Standard Platform challenge schemas."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    status: str = "ok"
    slug: str
    version: str


class VersionResponse(BaseModel):
    api_version: str
    challenge_version: str
    sdk_version: str
    capabilities: list[str]


class WeightsResponse(BaseModel):
    challenge_slug: str
    epoch: int | None = None
    weights: dict[str, float]
    metadata: dict[str, Any] = Field(default_factory=dict)
    computed_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
