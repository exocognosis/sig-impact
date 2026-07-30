# Security policy

## Supported versions

`sig-impact` is a deterministic modeling tool and performs no cryptographic
operations. Security and correctness fixes are applied to the latest release
and the `main` branch.

## Reporting a vulnerability

Please do not open a public issue for a suspected vulnerability. Use GitHub's
private vulnerability reporting feature:

1. open the repository's **Security** tab;
2. select **Advisories**;
3. select **Report a vulnerability**.

Include the affected version or commit, reproduction steps, expected behavior,
and actual behavior. Reports will be acknowledged as soon as practical.

For ordinary calculation errors, documentation problems, or incorrect scheme
constants that do not create a security risk, open a normal GitHub issue.

## Scope boundary

The output is for sizing and research. It must not be interpreted as a
cryptographic security proof, performance benchmark, consensus simulation, or
endorsement of an aggregation construction.
