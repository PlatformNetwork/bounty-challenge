"""FastAPI app factory for Platform challenge repositories."""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from contextlib import asynccontextmanager
from time import time

from fastapi import APIRouter, Depends, FastAPI

from .auth import build_internal_auth_dependency
from .config import ChallengeSettings
from .db import Database
from .schemas import HealthResponse, VersionResponse, WeightsResponse

GetWeightsFn = Callable[[], Awaitable[dict[str, float]]]


def create_challenge_app(
    *,
    settings: ChallengeSettings,
    database: Database,
    public_router: APIRouter,
    get_weights_fn: GetWeightsFn,
) -> FastAPI:
    """Create a complete FastAPI challenge app with standard Platform routes."""

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        await database.init()
        yield
        await database.close()

    app = FastAPI(title=settings.name, version=settings.version, lifespan=lifespan)

    @app.get("/health", response_model=HealthResponse, include_in_schema=False)
    async def health() -> HealthResponse:
        return HealthResponse(slug=settings.slug, version=settings.version)

    @app.get("/version", response_model=VersionResponse, include_in_schema=False)
    async def version() -> VersionResponse:
        return VersionResponse(
            api_version=settings.api_version,
            challenge_version=settings.version,
            sdk_version=settings.sdk_version,
            capabilities=[
                "get_weights",
                "proxy_routes",
                "sqlite",
                "manual_bounties",
                "manual_emissions",
            ],
        )

    internal_router = APIRouter(
        prefix="/internal/v1",
        dependencies=[Depends(build_internal_auth_dependency(settings))],
    )

    @internal_router.get("/get_weights", response_model=WeightsResponse)
    async def get_weights() -> WeightsResponse:
        weights = await get_weights_fn()
        return WeightsResponse(
            challenge_slug=settings.slug,
            epoch=int(time()),
            weights=weights,
        )

    app.include_router(internal_router)
    app.include_router(public_router)
    return app
