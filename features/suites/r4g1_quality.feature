@status:enforced
Feature: R4G1 generated-response quality
  The active R4G1 path must reject pathological text before it reaches the browser.

  Scenario: reject the repetitive geometric response to hello
    Given the R4G1 runtime returned the browser's repetitive hello response
    When the server validates the generated response
    Then the response is rejected as unusable

  Scenario: accept a concise readable response to hello
    Given the R4G1 runtime returned a concise readable hello response
    When the server validates the generated response
    Then the response is accepted as usable
