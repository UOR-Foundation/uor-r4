default:
    @just --list

# Run the Cucumber/Gherkin behavior suite.
bdd:
    cargo test --test bdd
