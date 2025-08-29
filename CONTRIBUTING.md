# Setting up pre-commit hooks

## Installing uv

Install the [uv][1] python package manager if you don't have it. You can also use [pip][3].

On macOS and Linux:
```
curl -LsSf https://astral.sh/uv/install.sh | sh
```

On Windows:
```
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

## Installing pre-commit

Install [pre-commit][2] with uv:

```sh
uv tool install pre-commit
```

Or install [pre-commit][2] with pip:

```sh
pip install pre-commit
```

## Installing pre-commit hooks

Install the pre-commit hooks with

```sh
pre-commit install
```

This ensures before a commit that tests run, formatting is applied, and clippy lints are checked.

[1]: https://github.com/astral-sh/uv
[2]: https://pre-commit.com/
[3]: https://pip.pypa.io/en/stable/installation/