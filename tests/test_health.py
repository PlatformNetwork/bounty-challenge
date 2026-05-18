from __future__ import annotations


async def test_health_and_version(client):
    health = await client.get("/health")
    assert health.status_code == 200
    assert health.json()["slug"] == "bounty-challenge"

    version = await client.get("/version")
    assert version.status_code == 200
    capabilities = version.json()["capabilities"]
    assert "get_weights" in capabilities
    assert "manual_bounties" in capabilities
    assert "manual_emissions" in capabilities


async def test_get_weights_requires_internal_auth(client):
    response = await client.get("/internal/v1/get_weights")
    assert response.status_code == 401

    bad_slug = await client.get(
        "/internal/v1/get_weights",
        headers={
            "Authorization": "Bearer test-token",
            "X-Platform-Challenge-Slug": "other",
        },
    )
    assert bad_slug.status_code == 400
