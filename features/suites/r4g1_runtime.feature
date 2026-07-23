@status:enforced
Feature: R4G1 runtime selection and fallback policy
  The browser actively tests R4G1 and must never silently fall back.

  Scenario: select R4G1 when no engine was saved
    Given the browser has no saved engine selection
    When the server resolves the synthesis engine
    Then the selected engine is R4G1

  Scenario: use legacy only when explicitly selected
    Given the browser explicitly selected the legacy engine
    When the server resolves the synthesis engine
    Then the selected engine is Legacy TLA/TLS

  Scenario: show R4G1 as the active browser option
    Then the browser UI selects R4G1 and does not offer automatic fallback

  Scenario: fail explicitly when R4G1 is unavailable
    Given the R4G1 runtime is unavailable
    When the R4G1 chat endpoint builds its unavailable response
    Then it returns HTTP 503 without invoking a fallback engine
