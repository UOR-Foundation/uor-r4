@status:enforced
Feature: Hologram/R4 formal monograph and implementation specification
  The formal monograph must consolidate all 12 formal analysis sections, maintain a traceability matrix mapping modules to proof obligations, and enforce explicit non-goal disavowals.

  Scenario: Validate full formal monograph section coverage and module links
    Given the formal monograph document "docs/hologram_r4_formal_monograph.md"
    When validated by the monograph traceability verifier
    Then all 12 sections and 9 implementation modules are verified
    And all 2 non-goal disavowals are present

  Scenario: Reject formal monograph missing a required section
    Given a monograph text missing "Section 10: Traceability Matrix"
    When validated by the monograph traceability verifier
    Then validation fails with a missing section error

  Scenario: Reject formal monograph missing explicit non-goal disavowals
    Given a monograph text missing the "No Human-Level Reasoning Claim" disavowal
    When validated by the monograph traceability verifier
    Then validation fails with a missing non-goal error
