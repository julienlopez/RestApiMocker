Feature: Internal API - Configuration
  The internal API exposes server configuration.

  Scenario: Get server configuration
    Given the backend is running
    When I request the server configuration
    Then the response status is 200
    And the configuration contains the public port
    And the configuration contains the private port
