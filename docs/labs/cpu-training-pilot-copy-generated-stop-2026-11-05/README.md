# Generated-trajectory copy stop gate

This record fits a geometric stop gate on the actual longest-first copy
trajectories from panel 2711 and evaluates it unchanged on panel 2712. The gate
uses the existing 256-bit sign-signature, eight-prototype-per-class Hamming
learner. The terminal label is the normalized reference length.

The main result is negative: on both artifact seeds the dev gate produced
**0/449** exact replies. It sometimes fired at the reference length, but those
bodies were not exact. Training-panel premature stops were **34** and **31**
against a 4-row 1% limit; dev premature stops were **3** and **8**.

The oracle ceiling remains 11/453 train and 13/449 dev. The current gate does
not recover it. This closes the fixed all-reference-length labeling with eight
signed prototypes; it does not close geometric termination generally.

Attempts 1–3 are preserved:

- attempt 1 used the wrong report schema literal;
- attempt 2 completed but wrote transient `/dev/fd` report paths;
- attempt 3 stopped after report reads were moved to hashes because a process
  substitution could not be read twice.

Attempt 4 is final and includes a byte-identical repeat after timing removal.
Published evidence is in [`evidence/`](evidence/).
