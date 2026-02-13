# Church-of-FEAR Ledger

An immutable, biophysically‑aware deed ledger for the Church-of-FEAR, where **deeds** are tracked as `DeedEvent`s with ecological and ethical invariants.

- CHURCH tokens mint on bioload reduction and ethical compliance.
- XR‑grid visualization uses Bevy to render Jetson‑Line trajectories.
- Compliance layer enforces ecological and ethics policies before mint.

## Running

```bash
RUST_LOG=info cargo run
Testing
bash
cargo test
text

***

## aln/compliance_policy.aln

```aln
# Church-of-FEAR Compliance Policy

POLICY_VERSION  = 1
ROH_MAX         = 0.3
DECAY_MAX       = 1.0
ALLOW_HARM      = FALSE

RULE ethics_guard:
  IF life_harm_flag == TRUE
  THEN reject_deed

RULE roh_decay_guard:
  IF roh > ROH_MAX OR decay > DECAY_MAX
  THEN reject_deed
