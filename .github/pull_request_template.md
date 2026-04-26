## Summary

- What changed:
- Why:

## Scope

- Primary crate (required):
- Secondary crates (optional, <= 2 recommended):
- Touches `zom-protocol`? (yes/no):

## Architecture Change (SoT)

- Changes dependency boundary? (yes/no):
- Changes crate ownership/responsibility? (yes/no):
- Updated SoT docs:
  - [ ] `docs/architecture/Crate边界契约.md`
  - [ ] `docs/architecture/架构哲学与模块宪章.md`
  - [ ] `docs/standards/开发规范手册.md` (if applicable)
  - [ ] `docs/playbooks/*` (if applicable)

## Risk and Rollback

- Risk:
- Rollback plan:

## Verification

- [ ] `./scripts/check-boundaries.sh`
- [ ] Relevant tests passed
- Extra checks:

## Checklist

- [ ] No cross-layer 1:1 mirrored types were introduced
- [ ] No UI-specific details leaked into core crates
- [ ] Boundary drift (if any) was fixed before feature stacking
