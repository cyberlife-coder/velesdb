## Description

Please include a summary of the changes and the related issue. Include relevant motivation and context.

Fixes # (issue)

## Type of Change

Please delete options that are not relevant.

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🔧 Refactoring (no functional changes)
- [ ] ⚡ Performance improvement

## How Has This Been Tested?

Please describe the tests that you ran to verify your changes. Provide instructions so we can reproduce.

- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual testing

**Test Configuration:**
- OS:
- Rust version:

## Checklist

- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Unsafe Code Checklist

> **Complete this section if your PR adds or modifies `unsafe` code.**
> Skip if no unsafe code is involved.

- [ ] All `unsafe fn` have `# Safety` documentation
- [ ] All `unsafe {}` blocks have `// SAFETY:` comments explaining why it's sound
- [ ] Invariants documented in [docs/SOUNDNESS.md](../docs/SOUNDNESS.md)
- [ ] No undefined behavior with valid inputs
- [ ] Edge cases tested (null, overflow, alignment, empty)
- [ ] Miri tests pass (if applicable): `cargo +nightly miri test <test_name>`

### Unsafe Justification

If adding new `unsafe`:
- [ ] Safe alternative was considered and rejected (explain why below)
- [ ] Performance benefit measured (if performance-motivated)

**Why is `unsafe` necessary?**
<!-- Explain why a safe alternative isn't feasible -->

**What are the invariants?**
<!-- List the conditions that must hold for this code to be sound -->

## Screenshots (if applicable)

Add screenshots to help explain your changes.

## Additional Notes

Add any other notes about the PR here.
