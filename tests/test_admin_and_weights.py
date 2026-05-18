from __future__ import annotations


async def test_owner_reviews_submission_and_rewards_it(client, owner_headers, internal_headers):
    await _create_project(client, owner_headers)
    submission = await _create_submission(client)
    submission_id = submission.json()["id"]

    review = await client.post(
        "/admin/reviews",
        headers=owner_headers,
        json={
            "submission_id": submission_id,
            "score": 94,
            "status": "accepted",
            "comments": "Best UX and clean implementation.",
        },
    )
    assert review.status_code == 200
    assert review.json()["score"] == 94

    reward = await client.post(
        f"/admin/submissions/{submission_id}/reward",
        headers=owner_headers,
        json={"weight": 2.5, "reason": "winner", "updated_by": "sudo-owner"},
    )
    assert reward.status_code == 200
    assert reward.json()["hotkey"] == "hotkey-a"
    assert reward.json()["weight"] == 2.5

    weights = await client.get("/internal/v1/get_weights", headers=internal_headers)
    assert weights.status_code == 200
    assert weights.json()["weights"] == {"hotkey-a": 2.5}

    leaderboard = await client.get("/leaderboard")
    assert leaderboard.status_code == 200
    assert leaderboard.json()[0]["reason"] == "winner"


async def test_owner_can_replace_manual_emissions(client, owner_headers, internal_headers):
    response = await client.post(
        "/admin/emissions",
        headers=owner_headers,
        json={
            "replace": True,
            "updated_by": "sudo-owner",
            "emissions": [
                {"hotkey": "hotkey-a", "weight": 1.0, "reason": "first"},
                {"hotkey": "hotkey-b", "weight": 0.5, "reason": "second"},
            ],
        },
    )
    assert response.status_code == 200
    assert [item["hotkey"] for item in response.json()] == ["hotkey-a", "hotkey-b"]

    response = await client.post(
        "/admin/emissions",
        headers=owner_headers,
        json={
            "replace": True,
            "emissions": [{"hotkey": "hotkey-c", "weight": 3.0}],
        },
    )
    assert response.status_code == 200

    weights = await client.get("/internal/v1/get_weights", headers=internal_headers)
    assert weights.json()["weights"] == {"hotkey-c": 3.0}


async def test_emission_validation_rejects_bad_weights(client, owner_headers):
    response = await client.post(
        "/admin/emissions",
        headers=owner_headers,
        json={"emissions": [{"hotkey": "hotkey-a", "weight": -1.0}]},
    )
    assert response.status_code == 422


async def test_owner_token_required(client):
    response = await client.post(
        "/admin/emissions",
        headers={"Authorization": "Bearer wrong"},
        json={"emissions": [{"hotkey": "hotkey-a", "weight": 1.0}]},
    )
    assert response.status_code == 401


async def test_project_deadline_blocks_late_submission(client, owner_headers):
    project = await client.post(
        "/admin/projects",
        headers=owner_headers,
        json={
            "project_id": "closed",
            "title": "Closed",
            "description": "Closed project",
            "deadline_at": "2000-01-01T00:00:00Z",
        },
    )
    assert project.status_code == 200

    response = await client.post(
        "/submissions",
        json={
            "project_id": "closed",
            "miner_hotkey": "hotkey-a",
            "github_url": "https://github.com/miner/work",
        },
    )
    assert response.status_code == 400
    assert "deadline" in response.text


async def _create_project(client, owner_headers):
    response = await client.post(
        "/admin/projects",
        headers=owner_headers,
        json={
            "project_id": "cli-ui",
            "title": "Best CLI UI/UX",
            "description": "Improve CLI UX",
        },
    )
    assert response.status_code == 200
    return response


async def _create_submission(client):
    response = await client.post(
        "/submissions",
        json={
            "project_id": "cli-ui",
            "miner_hotkey": "hotkey-a",
            "github_url": "https://github.com/miner/work",
        },
    )
    assert response.status_code == 200
    return response
