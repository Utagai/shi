# Contributing changes

Please [open an issue](https://github.com/Utagai/shi/issues/new) for your feature request or problem. If you would like to contribute modifications, you can then open a PR that references and addresses that issue.

# Setting up pre-commit hooks

Please follow [pre-commit](https://pre-commit.com/#install) installation instructions.

Then in the repository, install the pre-commit hooks with

```sh
pre-commit install
```

This ensures before a commit that tests run, formatting is applied, and clippy lints are checked.