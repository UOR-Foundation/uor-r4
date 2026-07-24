@status:enforced
Feature: Maintain a PDF-to-implementation traceability matrix for the formal direction
  The living PDF traceability matrix must map all 15 formal direction sections to issues, code locations, and verified evidence artifacts.

  Scenario: Validate full PDF section mapping completeness and evidence links
    Given the living PDF traceability matrix
    When audited by the PDF traceability verifier
    Then all 15 sections are mapped to valid code locations and evidence artifacts
    And the audit report certification status is verified

  Scenario: Reject traceability matrix entry with invalid claim class
    Given a traceability row with invalid claim class "HypotheticalSpec"
    When audited by the PDF traceability verifier
    Then validation fails with an invalid claim class error
