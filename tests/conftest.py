from __future__ import annotations

import os
import tempfile
from pathlib import Path

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import delete

_TEST_DIR = Path(tempfile.mkdtemp(prefix="bounty-challenge-tests-"))
_TEST_DB = _TEST_DIR / "challenge.sqlite3"

os.environ.setdefault("CHALLENGE_DATABASE_URL", f"sqlite+aiosqlite:///{_TEST_DB}")
os.environ.setdefault("CHALLENGE_SHARED_TOKEN", "test-token")
os.environ.setdefault("CHALLENGE_OWNER_TOKEN", "owner-token")

from bounty_challenge.app import app  # noqa: E402
from bounty_challenge.db import database  # noqa: E402
from bounty_challenge.models import Emission, Project, Review, Submission  # noqa: E402


@pytest.fixture(scope="session", autouse=True)
async def initialized_database():
    await database.init()
    yield
    await database.close()


@pytest.fixture(autouse=True)
async def clean_database(initialized_database):
    async with database.engine.begin() as connection:
        await connection.execute(delete(Review))
        await connection.execute(delete(Emission))
        await connection.execute(delete(Submission))
        await connection.execute(delete(Project))


@pytest.fixture
async def client():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://testserver") as async_client:
        yield async_client


@pytest.fixture
def owner_headers() -> dict[str, str]:
    return {"Authorization": "Bearer owner-token"}


@pytest.fixture
def internal_headers() -> dict[str, str]:
    return {
        "Authorization": "Bearer test-token",
        "X-Platform-Challenge-Slug": "bounty-challenge",
    }
