Feature: Path Parameter Mocks
  Mocks can use path patterns with wildcard parameters.

  Scenario: Wildcard path parameter matches any value
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "user found"
    And I send a GET request to the public server at "/users/42"
    Then the public response status is 200
    And the public response body is "user found"

  Scenario: Wildcard matches different values
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "user found"
    And I send a GET request to the public server at "/users/999"
    Then the public response status is 200
    And the public response body is "user found"

  Scenario: Extra path segments do not match
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "user found"
    And I send a GET request to the public server at "/users/42/profile"
    Then the public response status is 404
    And the public response body contains "No mock configured"

  Scenario: Multiple path parameters
    Given the backend is running
    When I register a mock for "GET" on "/users/:uid/posts/:pid" with status 200 and body "post found"
    And I send a GET request to the public server at "/users/5/posts/17"
    Then the public response status is 200
    And the public response body is "post found"
