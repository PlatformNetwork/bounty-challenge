"""Platform-compatible challenge settings."""

from __future__ import annotations

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class ChallengeSettings(BaseSettings):
    """Runtime settings for Bounty Challenge."""

    model_config = SettingsConfigDict(env_prefix="CHALLENGE_", extra="ignore")

    slug: str = "bounty-challenge"
    name: str = "Bounty Challenge"
    version: str = "0.1.0"
    api_version: str = "1.0"
    sdk_version: str = "1.0.0"
    database_url: str = "sqlite+aiosqlite:////data/bounty-challenge.sqlite3"
    artifact_root: str = "/data/submissions"
    shared_token: str | None = Field(default=None, repr=False)
    shared_token_file: str | None = Field(
        default="/run/secrets/platform/challenge_token",
        repr=False,
    )
    owner_token: str | None = Field(default=None, repr=False)
    owner_token_file: str | None = Field(
        default="/run/secrets/platform/owner_token",
        repr=False,
    )
    host: str = "0.0.0.0"
    port: int = 8000
    leaderboard_limit: int = 100
