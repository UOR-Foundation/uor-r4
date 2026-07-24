@status:enforced
Feature: Shared node space with multi-edge algebras
  A single canonical node identity can participate in multiple edge algebras (semantic, causal, temporal, constraint, goal-progress, evidence, refinement, forward, reverse) without node duplication.

  Scenario: Register multiple edge algebras on shared node identities
    Given a shared node graph with 5 canonical node identities
    When edges of kind Semantic, Causal, and Evidence are added between node 0 and node 1
    Then node 0 has 3 outgoing edges
    And the graph contains 1 edge for each of the 3 edge kinds

  Scenario: Pack and unpack multi-edge structures into 16-byte binary layouts
    Given a packed multi-edge between node 12 and node 34 with kind Causal and weight 256
    When the multi-edge is serialized to 16 bytes and deserialized
    Then the deserialized multi-edge preserves source node 12, destination node 34, kind Causal, and weight 256

  Scenario: Reject causal cycles violating the Directed Acyclic Graph (DAG) requirement
    Given a shared node graph with 3 nodes
    When a causal cycle 0 -> 1 -> 2 -> 0 is added
    Then graph validation fails with a causal cycle error

  Scenario: Reject dangling node references outside graph capacity
    Given a packed multi-edge referencing destination node 99 in a graph of size 10
    When the multi-edge is validated against the node capacity
    Then validation fails with a dangling node reference error
