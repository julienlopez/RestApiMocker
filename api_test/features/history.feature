Feature: Request History
  The backend records all requests hitting the public server.

  Scenario: History is initially empty
    Given the backend is running
    When I request the history
    Then the response status is 200
    And the history is empty

  Scenario: Requests are recorded in history
    Given the backend is running
    When I send a GET request to the public server at "/hello"
    And I request the history
    Then the response status is 200
    And the history contains an entry with method "GET" and path "/hello"

  Scenario: Multiple requests are recorded
    Given the backend is running
    When I send a GET request to the public server at "/one"
    And I send a POST request to the public server at "/two"
    And I request the history
    Then the response status is 200
    And the history has at least 2 entries
