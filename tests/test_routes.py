from __future__ import annotations

from fastapi.routing import APIRoute

from bounty_challenge.app import app


async def test_admin_creates_project_and_miner_submits_github_work(client, owner_headers):
    unauthorized = await client.post(
        "/admin/projects",
        json={
            "project_id": "cli-ui",
            "title": "Best CLI UI/UX",
            "description": "Improve the CLI interface",
        },
    )
    assert unauthorized.status_code == 503 or unauthorized.status_code == 401

    project = await client.post(
        "/admin/projects",
        headers=owner_headers,
        json={
            "project_id": "cli-ui",
            "title": "Best CLI UI/UX",
            "description": "Improve the CLI interface",
            "goals": "Make the CLI clearer, faster, and more pleasant to use.",
            "repository_url": "https://github.com/PlatformNetwork/example-cli",
            "metadata": {"rubric": ["visual", "flow", "accessibility"]},
        },
    )
    assert project.status_code == 200
    assert project.json()["project_id"] == "cli-ui"

    projects = await client.get("/projects")
    assert projects.status_code == 200
    assert projects.json()[0]["metadata"]["rubric"] == ["visual", "flow", "accessibility"]

    submission = await client.post(
        "/submissions",
        json={
            "project_id": "cli-ui",
            "miner_hotkey": "hotkey-a",
            "name": "polished-cli",
            "github_url": "https://github.com/miner/polished-cli",
            "branch": "main",
            "description": "New navigation and theme.",
        },
    )
    assert submission.status_code == 200
    payload = submission.json()
    assert payload["project_id"] == "cli-ui"
    assert payload["miner_hotkey"] == "hotkey-a"
    assert payload["status"] == "submitted"

    duplicate = await client.post(
        "/submissions",
        json={
            "project_id": "cli-ui",
            "miner_hotkey": "hotkey-a",
            "name": "polished-cli",
            "github_url": "https://github.com/miner/polished-cli",
            "branch": "main",
        },
    )
    assert duplicate.status_code == 409


async def test_submission_rejects_non_github_url(client, owner_headers):
    await _create_project(client, owner_headers)

    response = await client.post(
        "/submissions",
        json={
            "project_id": "demo",
            "miner_hotkey": "hotkey-a",
            "github_url": "https://gitlab.com/miner/work",
        },
    )

    assert response.status_code == 422


async def test_public_routes_are_decorated_for_proxy_discovery():
    public_paths = {
        route.path
        for route in app.routes
        if isinstance(route, APIRoute)
        and getattr(route.endpoint, "__platform_public_route__", False)
    }

    assert "/projects" in public_paths
    assert "/submissions" in public_paths
    assert "/leaderboard" in public_paths
    assert "/internal/v1/get_weights" not in public_paths


async def _create_project(client, owner_headers):
    return await client.post(
        "/admin/projects",
        headers=owner_headers,
        json={
            "project_id": "demo",
            "title": "Demo",
            "description": "Demo project",
        },
    )
