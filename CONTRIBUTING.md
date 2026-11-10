# Contributing to Cylixin

We welcome contributions to the Cylixin project! Whether you're interested in reporting bugs, suggesting new features, or contributing code, your help is greatly appreciated.

## How to Contribute

### Reporting Bugs

If you encounter a bug in Cylixin, please help us by submitting a detailed bug report. When reporting a bug, please include:

* **A clear and descriptive title** for the issue.
* **Steps to reproduce the bug**, ideally a short `.cyx` program that shows it.
* **The expected behavior** and what actually happened.
* **Your operating system and any relevant environment details** (for example, your LLVM and clang versions).
* **Any error messages or screenshots** that might be helpful.

Please check the [issue tracker](https://github.com/cylixin/cylixin/issues) to see if the bug has already been reported. If so, feel free to add any additional information to the existing issue.

### Suggesting Features

We're always open to ideas for new features and improvements to Cylixin. If you have a suggestion, please open a new issue with:

* **A clear and descriptive title** for the feature request.
* **A detailed explanation** of the proposed feature and its benefits.
* **Use cases** or scenarios where this feature would be valuable.
* **Any potential drawbacks or considerations.**

### Picking Up an Issue

Looking for something to work on? Start with issues labelled [`good first issue`](https://github.com/cylixin/cylixin/labels/good%20first%20issue), or see the roadmap in [#33](https://github.com/cylixin/cylixin/issues/33).

To claim an issue, comment on it with:

```
/assign-me
```

A bot assigns the issue to you and moves it from `status: to do` to `status: in progress`, so others know it's taken. You can only claim issues that are open and not already assigned to someone else.

Maintainers can assign someone else with `/assign @username`. GitHub only allows assigning people who have commented on the issue (or who are collaborators), so ask that person to comment first.

If you can no longer work on an issue, say so in a comment so it can be freed up for someone else.

## Contributing Code

Cylixin is in its early stages, so for anything bigger than a small fix, please **open or comment on an issue first** to agree on the approach. That avoids duplicated effort and work that doesn't fit the project's direction.

### Workflow

1. **Claim the issue** with `/assign-me` (see above).
2. **Fork the repository** to your own GitHub account.
3. **Create a branch** with a descriptive name, for example `fix/const-reassign` or `feature/math-builtins`.
4. **Make your changes**, with tests where it makes sense (see [Automated Checks](#automated-checks)).
5. **Commit** with clear messages. We use the style `type: short description`, for example `fix: reject assignment to const`. Common types are `fix`, `feat`, `docs`, `test`, `ci`, `chore`, `style` and `refactor`.
6. **Open a pull request** against `main`. Fill in the template, and link the issue with `Closes #123` so it closes automatically when merged.

Keep each pull request focused on one issue. Small pull requests get reviewed much faster than large ones.

### Automated Checks

Every pull request runs CI automatically:

| Check | What it verifies |
|---|---|
| **Formatting** | The code is formatted with `cargo fmt` |
| **Build, lint and test** | The compiler builds, `cargo clippy` reports no warnings, `cargo test` passes, and every file in `cylixin_compiler/code_examples/` compiles (`demo.cyx` is also run) |
| **All checks passed** | All of the above succeeded. This is the check that must be green before merging. |

When a check fails, the problem lines are annotated in your pull request's **Files changed** tab, and the run's summary page lists the command that reproduces it. **Please fix failing checks before asking for review.** Maintainers review pull requests once they're green.

Run the same checks locally before pushing (from `cylixin_compiler/`):

```bash
cargo fmt                                   # formatting (CI runs cargo fmt --check)
cargo clippy --all-targets -- -D warnings   # lints: any warning fails CI
cargo test                                  # unit tests
```

Or run exactly what CI runs, from the repository root:

```bash
.github/scripts/rust-check.sh fmt      # also: clippy, test, examples
```

### Review and Merging

`main` is protected. A pull request can only be merged when:

1. **All checks passed** is green.
2. **A maintainer has approved it.** Approvals only count from people with write access to the repository.
3. **The approval is for the latest version.** Pushing new commits dismisses earlier approvals, so the final code is always what was reviewed.
4. **All review conversations are resolved.**

What to expect:

* A maintainer will review your pull request once its checks pass. If nobody has responded after a few days, it's fine to leave a polite comment asking for a review.
* Reviewers may request changes. Push new commits to the same branch to address them, then resolve each conversation once it's handled, or reply if you disagree.
* Once approved, a maintainer merges the pull request. You don't need to do anything else.

## Documentation

Documentation lives in the [`docs/`](docs) folder of this repository:

* [`docs/syntax.md`](docs/syntax.md): the language as it's implemented today
* [`docs/architecture.md`](docs/architecture.md): how the compiler is put together
* [`docs/design_principles.md`](docs/design_principles.md): the long-term direction

Documentation changes go through the same pull request process as code. If your change adds or alters language behavior, please update `docs/syntax.md` in the same pull request.

## Code of Conduct

We expect all contributors to be respectful and considerate of others. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions and Discussion

For questions, ideas and general discussion, use [GitHub Discussions](https://github.com/cylixin/cylixin/discussions). For anything about a specific issue or pull request, comment there instead.

Thank you for your interest in contributing to Cylixin! Your help is invaluable.
