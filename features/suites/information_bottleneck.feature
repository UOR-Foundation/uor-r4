@status:enforced
Feature: Information-Bottleneck and predictive entropy compiler objectives
  The offline compiler objective must evaluate predictive entropy H(A|R), future state entropy H(S_future|R), and Information-Bottleneck cost min I(Z;X) - beta * I(Z;Y_future) with split-safe held-out validation reporting.

  Scenario: Evaluate Shannon entropy of probability distributions
    Given a uniform 4-state probability distribution [0.25, 0.25, 0.25, 0.25]
    When its Shannon entropy is evaluated
    Then the entropy equals the natural logarithm of 4
    And a deterministic 1-state distribution has zero entropy

  Scenario: Estimate Information-Bottleneck mutual information terms
    Given a diagonal joint probability matrix representing dependent variables Z and X
    When mutual information I(Z;X) is estimated
    Then the mutual information equals natural logarithm of 2
    And independent variables yield zero mutual information

  Scenario: Audit region topological split decisions on held-out data
    Given an Information-Bottleneck configuration with beta 1.5
    And baseline composite objective scores on training and held-out evaluation sets
    When a region split candidate reduces held-out composite score J
    Then the region decision auditor selects the Split action
    And the decision report includes training and held-out objective component breakdowns
