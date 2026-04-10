Feature: Basic Mock Registration and Matching
  Users can register mocks and the public server returns them.

  Scenario: No mock configured returns fallback
    Given the backend is running
    When I send a GET request to the public server at "/unknown"
    Then the public response status is 404
    And the public response body contains "No mock configured"

  Scenario: Register and hit a simple mock
    Given the backend is running
    When I register a mock for "GET" on "/hello" with status 200 and body "Hi there"
    And I send a GET request to the public server at "/hello"
    Then the public response status is 200
    And the public response body is "Hi there"

  Scenario: Mock with different status code
    Given the backend is running
    When I register a mock for "GET" on "/error" with status 503 and body "service down"
    And I send a GET request to the public server at "/error"
    Then the public response status is 503
    And the public response body is "service down"

  Scenario: Method mismatch does not match mock
    Given the backend is running
    When I register a mock for "POST" on "/submit" with status 201 and body "created"
    And I send a GET request to the public server at "/submit"
    Then the public response status is 404
    And the public response body contains "No mock configured"

  Scenario: Mock list contains registered mocks
    Given the backend is running
    When I register a mock for "GET" on "/test" with status 200 and body "ok"
    And I request the mocks list
    Then the response status is 200
    And the mocks list is not empty
