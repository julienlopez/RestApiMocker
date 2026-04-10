"""
Shared step definitions used across all feature files.

pytest-bdd discovers these automatically when placed in conftest.py,
but we keep them in a dedicated file for clarity and import them in conftest.
"""

import requests as http
from pytest_bdd import given, when, then, parsers


# ── Given ──────────────────────────────────────────────────────────────────


@given("the backend is running")
def backend_is_running(backend):
    """The ``backend`` fixture already guarantees the server is up."""
    return backend


# ── When: public server requests ───────────────────────────────────────────


@when(
    parsers.parse('I send a GET request to the public server at "{path}"'),
    target_fixture="public_response",
)
def send_get_to_public(public_url, path):
    return http.get(f"{public_url}{path}")


@when(
    parsers.parse('I send a POST request to the public server at "{path}"'),
    target_fixture="public_response",
)
def send_post_to_public(public_url, path):
    return http.post(f"{public_url}{path}")


# ── When: mock registration ───────────────────────────────────────────────


@when(
    parsers.parse(
        'I register a mock for "{method}" on "{pattern}" with status {status:d} and body "{body}"'
    ),
    target_fixture="_mock_registered",
)
def register_mock(internal_url, method, pattern, status, body):
    payload = {
        "method": method,
        "path_pattern": pattern,
        "status": status,
        "body": body,
    }
    r = http.post(f"{internal_url}/internal/mock", json=payload)
    assert r.status_code == 200, f"Failed to register mock: {r.text}"
    return r


# ── Then: internal response assertions ─────────────────────────────────────


@then(parsers.parse("the response status is {code:d}"))
def response_status(response, code):
    assert response.status_code == code, (
        f"Expected {code}, got {response.status_code}: {response.text}"
    )


# ── Then: public response assertions ──────────────────────────────────────


@then(parsers.parse("the public response status is {code:d}"))
def public_response_status(public_response, code):
    assert public_response.status_code == code, (
        f"Expected {code}, got {public_response.status_code}: {public_response.text}"
    )


@then(parsers.parse('the public response body is "{body}"'))
def public_response_body_exact(public_response, body):
    assert public_response.text == body, (
        f"Expected body '{body}', got '{public_response.text}'"
    )


@then(parsers.parse('the public response body contains "{text}"'))
def public_response_body_contains(public_response, text):
    assert text in public_response.text, (
        f"Expected '{text}' in body, got '{public_response.text}'"
    )


# ── Helpers ────────────────────────────────────────────────────────────────


def _table_to_dicts(datatable):
    """Convert pytest-bdd's list-of-lists datatable into a list of dicts."""
    headers = datatable[0]
    return [dict(zip(headers, row)) for row in datatable[1:]]


# ── When: mock registration with conditions ───────────────────────────────


@when(
    parsers.parse(
        'I register a mock for "{method}" on "{pattern}" with status {status:d} and body "{body}" with conditions:'
    ),
    target_fixture="_mock_registered",
)
def register_mock_with_conditions(internal_url, method, pattern, status, body, datatable):
    conditions = []
    for row in _table_to_dicts(datatable):
        source = row["source"]
        key = row["key"]
        matcher_type = row["matcher_type"]
        matcher_value = row["matcher_value"]
        if matcher_type == "Exact":
            matcher = {"Exact": matcher_value}
        else:
            matcher = {"Regex": matcher_value}
        conditions.append({"source": source, "key": key, "matcher": matcher})

    payload = {
        "method": method,
        "path_pattern": pattern,
        "status": status,
        "body": body,
        "conditions": conditions,
    }
    r = http.post(f"{internal_url}/internal/mock", json=payload)
    assert r.status_code == 200, f"Failed to register mock: {r.text}"
    return r


# ── When: public server with headers ─────────────────────────────────────


@when(
    parsers.parse(
        'I send a GET request to the public server at "{path}" with headers:'
    ),
    target_fixture="public_response",
)
def send_get_with_headers(public_url, path, datatable):
    headers = {row["key"]: row["value"] for row in _table_to_dicts(datatable)}
    return http.get(f"{public_url}{path}", headers=headers)


# ── Then: internal (generic) response body assertions ─────────────────────


@then(parsers.parse('the response body is "{body}"'))
def response_body_exact(response, body):
    assert response.text == body, (
        f"Expected body '{body}', got '{response.text}'"
    )


@then(parsers.parse('the response body contains "{text}"'))
def response_body_contains(response, text):
    assert text in response.text, (
        f"Expected '{text}' in body, got '{response.text}'"
    )


# ── When: mock management ─────────────────────────────────────────────────


@when("I request the mocks list", target_fixture="response")
def request_mocks_list(internal_url):
    return http.get(f"{internal_url}/internal/mocks")


@when(parsers.parse("I delete mock at index {index:d}"), target_fixture="response")
def delete_mock_at_index(internal_url, index):
    return http.delete(f"{internal_url}/internal/mock/{index}")


@when("I delete all mocks", target_fixture="response")
def delete_all_mocks(internal_url):
    return http.delete(f"{internal_url}/internal/mocks")


@when(parsers.parse('I delete mocks by path pattern "{pattern}"'), target_fixture="response")
def delete_mocks_by_pattern(internal_url, pattern):
    return http.delete(
        f"{internal_url}/internal/mocks/by-pattern",
        params={"path_pattern": pattern},
    )


# ── Then: mocks list assertions ───────────────────────────────────────────


@then("the mocks list is not empty")
def mocks_list_not_empty(response):
    assert len(response.json()) > 0


@then("the mocks list is empty")
def mocks_list_is_empty(response):
    assert response.json() == []


@then(parsers.parse("the mocks list has {count:d} entry"))
def mocks_list_has_n_entries(response, count):
    assert len(response.json()) == count, (
        f"Expected {count} mocks, got {len(response.json())}"
    )


@then(parsers.parse('mock at index {index:d} has path pattern "{pattern}"'))
def mock_at_index_has_pattern(response, index, pattern):
    mocks = response.json()
    assert index < len(mocks), f"Index {index} out of range (have {len(mocks)})"
    assert mocks[index]["path_pattern"] == pattern, (
        f"Expected pattern '{pattern}', got '{mocks[index]["path_pattern"]}'"
    )
