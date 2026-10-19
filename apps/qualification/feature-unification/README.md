# Adversarial feature unification

`make test-portable-signatures` runs the existing portable signature and
revocation suites here while an independent consumer enables Dalek's
`legacy_compatibility` feature. Auth must still reject noncanonical scalars,
weak points and invalid signatures, preserving its budget and replay behavior.

All cryptographic API imports use IC Platform. The direct Dalek dev dependency
exists solely to provoke feature unification; it is excluded from every deployed
library graph. The harness is unpublished and reuses the canonical test sources.
It supplies no compatibility reader or alternate authentication implementation.
