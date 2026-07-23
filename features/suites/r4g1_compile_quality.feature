@status:enforced
Feature: R4G1 compilation quality gates
  Compilation must fail clearly when inputs are missing or quality is below baseline.

  Scenario: report missing corpus metadata
    Given the configured corpus metadata path is missing
    When R4G1 compilation inputs are validated
    Then compilation fails with the missing metadata error

  Scenario: reject a graph below the TLA baseline
    Given a graph quality report below the TLA baseline
    When the R4G1 quality gate validates the report
    Then the quality gate rejects the graph below baseline

  Scenario: accept a graph at the TLA baseline
    Given a graph quality report at or above the TLA baseline
    When the R4G1 quality gate validates the report
    Then the quality gate accepts the graph
