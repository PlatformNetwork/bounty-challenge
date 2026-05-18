"""Challenge authentication helpers."""

from __future__ import annotations

import secrets
from pathlib import Path

from fastapi import Header, HTTPException, status

from .config import ChallengeSettings


def read_token(value: str | None, file_path: str | None) -> str | None:
    if value:
        return value
    if not file_path:
        return None
    path = Path(file_path)
    if not path.is_file():
        return None
    token = path.read_text(encoding="utf-8").strip()
    return token or None


def build_internal_auth_dependency(settings: ChallengeSettings):
    async def verify_internal_auth(
        authorization: str | None = Header(default=None),
        challenge_slug: str | None = Header(default=None, alias="X-Platform-Challenge-Slug"),
    ) -> None:
        expected_token = read_token(settings.shared_token, settings.shared_token_file)
        if expected_token is None:
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail="Internal challenge token is not configured",
            )
        if authorization is None or not secrets.compare_digest(
            authorization, f"Bearer {expected_token}"
        ):
            raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Invalid internal authorization")
        if challenge_slug != settings.slug:
            raise HTTPException(status.HTTP_400_BAD_REQUEST, "Invalid challenge slug header")

    return verify_internal_auth


def build_owner_auth_dependency(settings: ChallengeSettings):
    async def verify_owner_auth(
        authorization: str | None = Header(default=None),
        owner_token: str | None = Header(default=None, alias="X-Owner-Token"),
    ) -> None:
        expected_token = read_token(settings.owner_token, settings.owner_token_file)
        if expected_token is None:
            raise HTTPException(
                status.HTTP_503_SERVICE_UNAVAILABLE,
                "Owner token is not configured",
            )
        provided = None
        if authorization and authorization.startswith("Bearer "):
            provided = authorization.removeprefix("Bearer ")
        elif owner_token:
            provided = owner_token
        if provided is None or not secrets.compare_digest(provided, expected_token):
            raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Invalid owner authorization")

    return verify_owner_auth
