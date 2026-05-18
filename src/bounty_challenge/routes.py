"""Public and owner routes for manual bounty projects."""

from __future__ import annotations

import hashlib
import json
import math
from datetime import UTC, datetime
from typing import Annotated, Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import desc, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from .config import settings
from .db import database
from .models import Emission, Project, Review, Submission
from .sdk.auth import build_owner_auth_dependency
from .sdk.decorators import public_route

router = APIRouter()
DatabaseSession = Annotated[AsyncSession, Depends(database.session_dependency)]
OwnerAuth = Annotated[None, Depends(build_owner_auth_dependency(settings))]


class ProjectUpsertRequest(BaseModel):
    project_id: str = Field(min_length=1, max_length=128, pattern=r"^[A-Za-z0-9_.-]+$")
    title: str = Field(min_length=1, max_length=200)
    description: str = Field(min_length=1)
    goals: str = ""
    repository_url: str | None = None
    assets_url: str | None = None
    status: str = "active"
    deadline_at: datetime | None = None
    metadata: dict[str, Any] = Field(default_factory=dict)

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in {"active", "paused", "completed", "cancelled"}:
            raise ValueError("invalid project status")
        return value


class ProjectResponse(BaseModel):
    id: int
    project_id: str
    title: str
    description: str
    goals: str
    repository_url: str | None
    assets_url: str | None
    status: str
    deadline_at: datetime | None
    metadata: dict[str, Any]
    created_at: datetime
    updated_at: datetime


class SubmissionRequest(BaseModel):
    project_id: str = Field(min_length=1, max_length=128)
    miner_hotkey: str = Field(min_length=1, max_length=128)
    name: str = Field(default="submission", min_length=1, max_length=160)
    github_url: str = Field(min_length=1)
    branch: str | None = Field(default=None, max_length=160)
    commit_sha: str | None = Field(default=None, max_length=128)
    description: str = ""
    submission_hash: str | None = Field(default=None, min_length=8, max_length=128)
    metadata: dict[str, Any] = Field(default_factory=dict)

    @field_validator("github_url")
    @classmethod
    def validate_github_url(cls, value: str) -> str:
        if not (
            value.startswith("https://github.com/") or value.startswith("https://www.github.com/")
        ):
            raise ValueError("github_url must be a GitHub HTTPS URL")
        return value


class SubmissionResponse(BaseModel):
    id: int
    project_id: str
    miner_hotkey: str
    name: str
    github_url: str
    branch: str | None
    commit_sha: str | None
    description: str
    submission_hash: str
    status: str
    metadata: dict[str, Any]
    review_score: float | None = None
    created_at: datetime
    updated_at: datetime


class ReviewRequest(BaseModel):
    submission_id: int
    score: float = Field(ge=0, le=100)
    status: str = "reviewed"
    comments: str = ""
    reviewer: str = "owner"

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in {"reviewed", "accepted", "rejected"}:
            raise ValueError("invalid review status")
        return value


class ReviewResponse(BaseModel):
    id: int
    submission_id: int
    reviewer: str
    score: float
    status: str
    comments: str
    created_at: datetime


class EmissionEntry(BaseModel):
    hotkey: str = Field(min_length=1, max_length=128)
    weight: float = Field(ge=0)
    reason: str = ""
    project_id: str | None = None
    submission_id: int | None = None

    @field_validator("weight")
    @classmethod
    def validate_weight(cls, value: float) -> float:
        if not math.isfinite(value):
            raise ValueError("weight must be finite")
        return value


class EmissionSetRequest(BaseModel):
    replace: bool = False
    updated_by: str = "owner"
    emissions: list[EmissionEntry]


class RewardSubmissionRequest(BaseModel):
    weight: float = Field(ge=0)
    reason: str = ""
    updated_by: str = "owner"

    @field_validator("weight")
    @classmethod
    def validate_weight(cls, value: float) -> float:
        if not math.isfinite(value):
            raise ValueError("weight must be finite")
        return value


class EmissionResponse(BaseModel):
    hotkey: str
    weight: float
    reason: str
    project_id: str | None
    submission_id: int | None
    updated_by: str
    updated_at: datetime


@public_route(tags=["projects"])
@router.get("/projects", response_model=list[ProjectResponse])
async def list_projects(session: DatabaseSession) -> list[ProjectResponse]:
    result = await session.execute(select(Project).order_by(desc(Project.created_at)))
    return [_project_response(project) for project in result.scalars().all()]


@public_route(tags=["projects"])
@router.get("/projects/{project_id}", response_model=ProjectResponse)
async def get_project(project_id: str, session: DatabaseSession) -> ProjectResponse:
    project = await _project_by_public_id(session, project_id)
    if project is None:
        raise HTTPException(status_code=404, detail="project not found")
    return _project_response(project)


@public_route(tags=["submissions"])
@router.post("/submissions", response_model=SubmissionResponse)
async def create_submission(
    request: SubmissionRequest,
    session: DatabaseSession,
) -> SubmissionResponse:
    project = await _project_by_public_id(session, request.project_id)
    if project is None:
        raise HTTPException(status_code=404, detail="project not found")
    if project.status != "active":
        raise HTTPException(status_code=400, detail="project is not active")
    if project.deadline_at and datetime.now(UTC) >= _ensure_aware(project.deadline_at):
        raise HTTPException(status_code=400, detail="project deadline has passed")

    submission_hash = request.submission_hash or _hash_submission(request)
    submission = Submission(
        project_id=project.id,
        miner_hotkey=request.miner_hotkey,
        name=request.name,
        github_url=request.github_url,
        branch=request.branch,
        commit_sha=request.commit_sha,
        description=request.description,
        submission_hash=submission_hash,
        metadata_json=json.dumps(request.metadata, separators=(",", ":")),
    )
    session.add(submission)
    try:
        await session.commit()
    except IntegrityError as exc:
        await session.rollback()
        raise HTTPException(status_code=409, detail="submission already exists") from exc
    await session.refresh(submission, attribute_names=["project", "reviews"])
    return _submission_response(submission)


@public_route(tags=["submissions"])
@router.get("/submissions", response_model=list[SubmissionResponse])
async def list_submissions(
    session: DatabaseSession,
    project_id: str | None = None,
) -> list[SubmissionResponse]:
    statement = (
        select(Submission)
        .options(selectinload(Submission.project), selectinload(Submission.reviews))
        .order_by(desc(Submission.created_at))
        .limit(200)
    )
    if project_id:
        project = await _project_by_public_id(session, project_id)
        if project is None:
            return []
        statement = statement.where(Submission.project_id == project.id)
    result = await session.execute(statement)
    return [_submission_response(submission) for submission in result.scalars().all()]


@public_route(tags=["submissions"])
@router.get("/submissions/{submission_id}", response_model=SubmissionResponse)
async def get_submission(submission_id: int, session: DatabaseSession) -> SubmissionResponse:
    submission = await _submission_by_id(session, submission_id)
    if submission is None:
        raise HTTPException(status_code=404, detail="submission not found")
    return _submission_response(submission)


@public_route(tags=["leaderboard"])
@router.get("/leaderboard", response_model=list[EmissionResponse])
async def leaderboard(session: DatabaseSession) -> list[EmissionResponse]:
    return await _emission_responses(session)


@router.post("/admin/projects", response_model=ProjectResponse)
async def admin_upsert_project(
    request: ProjectUpsertRequest,
    session: DatabaseSession,
    _auth: OwnerAuth,
) -> ProjectResponse:
    project = await _project_by_public_id(session, request.project_id)
    if project is None:
        project = Project(project_id=request.project_id)
        session.add(project)
    project.title = request.title
    project.description = request.description
    project.goals = request.goals
    project.repository_url = request.repository_url
    project.assets_url = request.assets_url
    project.status = request.status
    project.deadline_at = request.deadline_at
    project.metadata_json = json.dumps(request.metadata, separators=(",", ":"))
    project.updated_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(project)
    return _project_response(project)


@router.post("/admin/reviews", response_model=ReviewResponse)
async def admin_review_submission(
    request: ReviewRequest,
    session: DatabaseSession,
    _auth: OwnerAuth,
) -> ReviewResponse:
    submission = await _submission_by_id(session, request.submission_id)
    if submission is None:
        raise HTTPException(status_code=404, detail="submission not found")
    review = Review(
        submission_id=submission.id,
        reviewer=request.reviewer,
        score=request.score,
        status=request.status,
        comments=request.comments,
    )
    submission.status = request.status
    session.add(review)
    await session.commit()
    await session.refresh(review)
    return _review_response(review)


@router.get("/admin/emissions", response_model=list[EmissionResponse])
async def admin_list_emissions(
    session: DatabaseSession,
    _auth: OwnerAuth,
) -> list[EmissionResponse]:
    return await _emission_responses(session)


@router.post("/admin/emissions", response_model=list[EmissionResponse])
async def admin_set_emissions(
    request: EmissionSetRequest,
    session: DatabaseSession,
    _auth: OwnerAuth,
) -> list[EmissionResponse]:
    if not request.emissions:
        raise HTTPException(status_code=400, detail="emissions cannot be empty")
    if request.replace:
        for existing in (await session.execute(select(Emission))).scalars().all():
            await session.delete(existing)
        await session.flush()
    for entry in request.emissions:
        emission = await session.get(Emission, entry.hotkey)
        if emission is None:
            emission = Emission(hotkey=entry.hotkey)
            session.add(emission)
        emission.weight = entry.weight
        emission.reason = entry.reason
        emission.project_id = entry.project_id
        emission.submission_id = entry.submission_id
        emission.updated_by = request.updated_by
        emission.updated_at = datetime.now(UTC)
    await session.commit()
    return await _emission_responses(session)


@router.post("/admin/submissions/{submission_id}/reward", response_model=EmissionResponse)
async def admin_reward_submission(
    submission_id: int,
    request: RewardSubmissionRequest,
    session: DatabaseSession,
    _auth: OwnerAuth,
) -> EmissionResponse:
    submission = await _submission_by_id(session, submission_id)
    if submission is None:
        raise HTTPException(status_code=404, detail="submission not found")
    submission.status = "rewarded"
    emission = await session.get(Emission, submission.miner_hotkey)
    if emission is None:
        emission = Emission(hotkey=submission.miner_hotkey)
        session.add(emission)
    emission.weight = request.weight
    emission.reason = request.reason
    emission.project_id = submission.project.project_id
    emission.submission_id = submission.id
    emission.updated_by = request.updated_by
    emission.updated_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(emission)
    return _emission_response(emission)


async def _project_by_public_id(session: AsyncSession, project_id: str) -> Project | None:
    return await session.scalar(select(Project).where(Project.project_id == project_id))


async def _submission_by_id(session: AsyncSession, submission_id: int) -> Submission | None:
    return await session.scalar(
        select(Submission)
        .where(Submission.id == submission_id)
        .options(selectinload(Submission.project), selectinload(Submission.reviews))
    )


async def _emission_responses(session: AsyncSession) -> list[EmissionResponse]:
    result = await session.execute(
        select(Emission).where(Emission.weight > 0).order_by(desc(Emission.weight))
    )
    return [_emission_response(emission) for emission in result.scalars().all()]


def _project_response(project: Project) -> ProjectResponse:
    return ProjectResponse(
        id=project.id,
        project_id=project.project_id,
        title=project.title,
        description=project.description,
        goals=project.goals,
        repository_url=project.repository_url,
        assets_url=project.assets_url,
        status=project.status,
        deadline_at=project.deadline_at,
        metadata=json.loads(project.metadata_json or "{}"),
        created_at=project.created_at,
        updated_at=project.updated_at,
    )


def _submission_response(submission: Submission) -> SubmissionResponse:
    score = max((review.score for review in submission.reviews), default=None)
    return SubmissionResponse(
        id=submission.id,
        project_id=submission.project.project_id,
        miner_hotkey=submission.miner_hotkey,
        name=submission.name,
        github_url=submission.github_url,
        branch=submission.branch,
        commit_sha=submission.commit_sha,
        description=submission.description,
        submission_hash=submission.submission_hash,
        status=submission.status,
        metadata=json.loads(submission.metadata_json or "{}"),
        review_score=score,
        created_at=submission.created_at,
        updated_at=submission.updated_at,
    )


def _review_response(review: Review) -> ReviewResponse:
    return ReviewResponse(
        id=review.id,
        submission_id=review.submission_id,
        reviewer=review.reviewer,
        score=review.score,
        status=review.status,
        comments=review.comments,
        created_at=review.created_at,
    )


def _emission_response(emission: Emission) -> EmissionResponse:
    return EmissionResponse(
        hotkey=emission.hotkey,
        weight=emission.weight,
        reason=emission.reason,
        project_id=emission.project_id,
        submission_id=emission.submission_id,
        updated_by=emission.updated_by,
        updated_at=emission.updated_at,
    )


def _hash_submission(request: SubmissionRequest) -> str:
    digest = hashlib.sha256()
    for value in (
        request.project_id,
        request.miner_hotkey,
        request.github_url,
        request.branch or "",
        request.commit_sha or "",
    ):
        digest.update(value.encode("utf-8"))
        digest.update(b"\0")
    return digest.hexdigest()


def _ensure_aware(value: datetime) -> datetime:
    if value.tzinfo is None:
        return value.replace(tzinfo=UTC)
    return value
