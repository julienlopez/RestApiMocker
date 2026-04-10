Feature: Conditional Mock Matching
  Mocks can have conditions on path parameters, query parameters, and headers.

  Scenario: Path param exact condition selects specific mock
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "Alice" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Exact        | 42            |
    And I register a mock for "GET" on "/users/:id" with status 200 and body "Default user"
    And I send a GET request to the public server at "/users/42"
    Then the public response status is 200
    And the public response body is "Alice"

  Scenario: Path param condition miss falls through to fallback
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "Alice" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Exact        | 42            |
    And I register a mock for "GET" on "/users/:id" with status 200 and body "Default user"
    And I send a GET request to the public server at "/users/99"
    Then the public response status is 200
    And the public response body is "Default user"

  Scenario: Query param exact condition
    Given the backend is running
    When I register a mock for "GET" on "/search" with status 200 and body "page 3 results" with conditions:
      | source     | key  | matcher_type | matcher_value |
      | QueryParam | page | Exact        | 3             |
    And I register a mock for "GET" on "/search" with status 200 and body "default results"
    And I send a GET request to the public server at "/search?page=3"
    Then the public response status is 200
    And the public response body is "page 3 results"

  Scenario: Query param condition miss falls to fallback
    Given the backend is running
    When I register a mock for "GET" on "/search" with status 200 and body "page 3 results" with conditions:
      | source     | key  | matcher_type | matcher_value |
      | QueryParam | page | Exact        | 3             |
    And I register a mock for "GET" on "/search" with status 200 and body "default results"
    And I send a GET request to the public server at "/search?page=1"
    Then the public response status is 200
    And the public response body is "default results"

  Scenario: Header exact condition
    Given the backend is running
    When I register a mock for "GET" on "/api" with status 200 and body "authorized" with conditions:
      | source | key           | matcher_type | matcher_value |
      | Header | X-Auth-Token  | Exact        | secret123     |
    And I register a mock for "GET" on "/api" with status 401 and body "unauthorized"
    And I send a GET request to the public server at "/api" with headers:
      | key          | value     |
      | X-Auth-Token | secret123 |
    Then the public response status is 200
    And the public response body is "authorized"

  Scenario: Header condition miss falls to fallback
    Given the backend is running
    When I register a mock for "GET" on "/api" with status 200 and body "authorized" with conditions:
      | source | key           | matcher_type | matcher_value |
      | Header | X-Auth-Token  | Exact        | secret123     |
    And I register a mock for "GET" on "/api" with status 401 and body "unauthorized"
    And I send a GET request to the public server at "/api"
    Then the public response status is 401
    And the public response body is "unauthorized"

  Scenario: Regex condition on path param
    Given the backend is running
    When I register a mock for "GET" on "/items/:id" with status 200 and body "numeric id" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Regex        | ^[0-9]+$      |
    And I register a mock for "GET" on "/items/:id" with status 200 and body "any id"
    And I send a GET request to the public server at "/items/123"
    Then the public response status is 200
    And the public response body is "numeric id"

  Scenario: Regex condition miss on path param
    Given the backend is running
    When I register a mock for "GET" on "/items/:id" with status 200 and body "numeric id" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Regex        | ^[0-9]+$      |
    And I register a mock for "GET" on "/items/:id" with status 200 and body "any id"
    And I send a GET request to the public server at "/items/abc"
    Then the public response status is 200
    And the public response body is "any id"

  Scenario: Multiple conditions must all match
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "specific" with conditions:
      | source     | key     | matcher_type | matcher_value |
      | PathParam  | id      | Exact        | 42            |
      | QueryParam | verbose | Exact        | true          |
    And I register a mock for "GET" on "/users/:id" with status 200 and body "fallback"
    And I send a GET request to the public server at "/users/42?verbose=true"
    Then the public response status is 200
    And the public response body is "specific"

  Scenario: Multiple conditions - one fails falls to fallback
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "specific" with conditions:
      | source     | key     | matcher_type | matcher_value |
      | PathParam  | id      | Exact        | 42            |
      | QueryParam | verbose | Exact        | true          |
    And I register a mock for "GET" on "/users/:id" with status 200 and body "fallback"
    And I send a GET request to the public server at "/users/42?verbose=false"
    Then the public response status is 200
    And the public response body is "fallback"

  Scenario: Most specific mock wins (more conditions first)
    Given the backend is running
    When I register a mock for "GET" on "/data/:id" with status 200 and body "fallback"
    And I register a mock for "GET" on "/data/:id" with status 200 and body "by-id" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Exact        | 1             |
    And I register a mock for "GET" on "/data/:id" with status 200 and body "by-id-and-fmt" with conditions:
      | source     | key | matcher_type | matcher_value |
      | PathParam  | id  | Exact        | 1             |
      | QueryParam | fmt | Exact        | json          |
    And I send a GET request to the public server at "/data/1?fmt=json"
    Then the public response status is 200
    And the public response body is "by-id-and-fmt"
