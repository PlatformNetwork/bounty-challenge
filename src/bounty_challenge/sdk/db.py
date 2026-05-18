"""Async SQLAlchemy helpers."""

from __future__ import annotations

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    """Base class for challenge-owned models."""


class Database:
    """Async database wrapper."""

    def __init__(self, database_url: str) -> None:
        self.engine = create_async_engine(
            database_url,
            connect_args={"check_same_thread": False} if database_url.startswith("sqlite") else {},
        )
        self._session_factory = async_sessionmaker(
            self.engine,
            expire_on_commit=False,
            autoflush=False,
        )

    async def init(self) -> None:
        async with self.engine.begin() as connection:
            if self.engine.url.get_backend_name().startswith("sqlite"):
                await connection.exec_driver_sql("PRAGMA journal_mode=WAL")
            await connection.run_sync(Base.metadata.create_all)

    async def close(self) -> None:
        await self.engine.dispose()

    @asynccontextmanager
    async def session(self) -> AsyncIterator[AsyncSession]:
        async with self._session_factory() as session:
            yield session

    async def session_dependency(self) -> AsyncIterator[AsyncSession]:
        async with self.session() as session:
            yield session
