"""Step definitions for the history feature."""

import requests as http
from pytest_bdd import scenarios, when, then, parsers

scenarios("../features/history.feature")


@when("I request the history", target_fixture="response")
def request_history(internal_url):
    return http.get(f"{internal_url}/internal/history")


@then("the history is empty")
def history_is_empty(response):
    assert response.json() == []


@then(
    parsers.parse('the history contains an entry with method "{method}" and path "{path}"')
)
def history_contains_entry(response, method, path):
    entries = response.json()
    assert any(
        e["method"] == method and e["path"] == path for e in entries
    ), f"No entry with method={method} path={path} in {entries}"


@then(parsers.parse("the history has at least {count:d} entries"))
def history_has_n_entries(response, count):
    assert len(response.json()) >= count
