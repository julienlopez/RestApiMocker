"""Step definitions for the configuration feature."""

import requests as http
from pytest_bdd import scenarios, when, then, parsers

scenarios("../features/config.feature")


@when("I request the server configuration", target_fixture="response")
def request_config(internal_url):
    return http.get(f"{internal_url}/internal/config")


@then("the configuration contains the public port")
def config_has_public_port(response, backend):
    data = response.json()
    assert data["public_port"] == backend.public_port


@then("the configuration contains the private port")
def config_has_private_port(response, backend):
    data = response.json()
    assert data["private_port"] == backend.private_port
