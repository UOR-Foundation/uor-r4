@status:enforced
Feature: Typed selective-prediction serving surfaces (legacy-coverage mode)
  The #838 typed status schema is served on every production surface in
  legacy-coverage mode: abstentions carry the typed cause, calibration
  presence fails closed, and no surface hides an abstention.

  @RF-30 @build
  Scenario: CLI abstention record carries the typed cause and coverage
    Given the deployed D4 policy abstains on a novel window
    When the CLI chat surface renders the turn
    Then the abstention record reads outcome "abstention" with cause and coverage "distributionally-novel" and no confidence value

  @RF-30 @build
  Scenario: native declined response carries the typed selective block
    Given a serving cascade whose R4G1 tier abstained
    When the native declined-by-all response is built
    Then it reports status "abstention" with cause "distributionally-novel" and null confidence and evidence

  @RF-30 @build
  Scenario: OpenAI-compatible abstention is a typed structured error
    Given a serving cascade whose R4G1 tier abstained
    When the OpenAI-compatible surface envelopes the decline
    Then the response is HTTP 422 with error type "uor_selective_prediction" and code "uor_abstention_distributionally_novel"
    And it is never an empty-choices success

  @RF-30 @build
  Scenario: streaming abstention terminates with a typed error event
    Given a streaming request whose cascade abstained
    When the stream frames are built
    Then no content chunk is emitted and the terminal frames are one typed error event then the DONE marker

  @RF-30 @build
  Scenario: present selective-calibration data fails closed
    Given an active bundle directory carrying a selective-calibration sidecar
    When a serving surface consults the bundle
    Then the outcome is hard-incompatibility with the typed envelope and never a legacy serve

  @RF-30 @build
  Scenario: wasm boundary returns typed values and never traps
    Given the wasm graph bundle is not installed
    When the typed wasm response surface is invoked
    Then it returns a typed hard-incompatibility value instead of trapping
