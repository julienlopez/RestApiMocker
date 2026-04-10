Feature: Delete Mocks
  Users can delete individual mocks by index or clear all mocks at once.

  Scenario: Delete a mock by index
    Given the backend is running
    When I register a mock for "GET" on "/a" with status 200 and body "alpha"
    And I register a mock for "GET" on "/b" with status 200 and body "beta"
    And I delete mock at index 0
    Then the response status is 200
    And the response body is "Mock deleted"
    When I request the mocks list
    Then the response status is 200
    And the mocks list has 1 entry
    And mock at index 0 has path pattern "/b"

  Scenario: Delete last remaining mock
    Given the backend is running
    When I register a mock for "GET" on "/only" with status 200 and body "alone"
    And I delete mock at index 0
    Then the response status is 200
    When I request the mocks list
    Then the response status is 200
    And the mocks list is empty

  Scenario: Delete with out-of-range index returns 404
    Given the backend is running
    When I register a mock for "GET" on "/x" with status 200 and body "x"
    And I delete mock at index 5
    Then the response status is 404
    And the response body contains "out of range"

  Scenario: Delete from empty registry returns 404
    Given the backend is running
    When I delete mock at index 0
    Then the response status is 404

  Scenario: Delete all mocks
    Given the backend is running
    When I register a mock for "GET" on "/a" with status 200 and body "a"
    And I register a mock for "GET" on "/b" with status 200 and body "b"
    And I register a mock for "GET" on "/c" with status 200 and body "c"
    And I delete all mocks
    Then the response status is 200
    And the response body is "All mocks deleted"
    When I request the mocks list
    Then the response status is 200
    And the mocks list is empty

  Scenario: Delete all on empty registry succeeds
    Given the backend is running
    When I delete all mocks
    Then the response status is 200
    When I request the mocks list
    Then the response status is 200
    And the mocks list is empty

  Scenario: Public server reflects deletion
    Given the backend is running
    When I register a mock for "GET" on "/temp" with status 200 and body "temporary"
    And I send a GET request to the public server at "/temp"
    Then the public response status is 200
    And the public response body is "temporary"
    When I delete mock at index 0
    Then the response status is 200
    When I send a GET request to the public server at "/temp"
    Then the public response status is 404
    And the public response body contains "No mock configured"

  Scenario: Delete mocks by path pattern removes all matching
    Given the backend is running
    When I register a mock for "GET" on "/users/:id" with status 200 and body "user fallback"
    And I register a mock for "GET" on "/users/:id" with status 200 and body "user 42" with conditions:
      | source    | key | matcher_type | matcher_value |
      | PathParam | id  | Exact        | 42            |
    And I register a mock for "GET" on "/posts/:id" with status 200 and body "post"
    And I delete mocks by path pattern "/users/:id"
    Then the response status is 200
    And the response body contains "2 mock(s) deleted"
    When I request the mocks list
    Then the response status is 200
    And the mocks list has 1 entry
    And mock at index 0 has path pattern "/posts/:id"

  Scenario: Delete by path pattern with no match returns 404
    Given the backend is running
    When I register a mock for "GET" on "/a" with status 200 and body "a"
    And I delete mocks by path pattern "/nonexistent"
    Then the response status is 404
    And the response body contains "No mocks found"

  Scenario: Delete by path pattern on empty registry returns 404
    Given the backend is running
    When I delete mocks by path pattern "/anything"
    Then the response status is 404
