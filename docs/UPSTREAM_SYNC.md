# Upstream synchronization

## Identity

- Upstream fetch remote: `https://gitlab.com/qemu-project/qemu.git`
- Baseline tag: `v11.1.3`
- Baseline commit: `005c61c0ef6f6ae615283440b4bb7be51b2c6ddd`
- ExR development branch: `main`
- Upstream push URL: `DISABLED` (prevents accidental writes to the official project)

No repository hosting origin is configured yet; this checkout itself is the
independent fork. The original upstream repository is never edited directly.

## Routine

1. `git fetch upstream --tags --prune`.
2. Inspect upstream release notes and `git log --no-merges <baseline>..<new-tag>`.
3. Merge or rebase one reviewed stable tag at a time on a synchronization branch;
   never rewrite the published ExR branch to make a conflict disappear.
4. Preserve `LICENSE`, `COPYING`, source copyright notices and third-party notices.
5. Review every ExR-owned file and public interface for changed semantics.
6. Re-run headless builds, i386/x86_64 TCG tests, available hardware-accelerated
   tests, configuration compatibility tests and real guest boots.
7. Update the recorded baseline only after tests pass. Record conflicts and
   deliberate deviations in `DECISIONS.md` and the relevant technical document.
8. Publish the result as a separate ExR synchronization commit/release.

Security fixes can be cherry-picked ahead of a full release merge when necessary,
but must identify the upstream commit and remain traceable during the next merge.
Submodule changes are reviewed explicitly; uninitialized submodule directories
are not assumed to contain tested firmware.
