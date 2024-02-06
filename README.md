<br /><br />

<div align="center">
  <h1 align="center">Changie</h1>
  <h4 align="center"> CLI for generating release notes in CHANGELOG's for substrate based projects maintained by the integrations tools team. </h4>
</div>

## Summary

This CLI helps generate release docs for a CHANGELOG. The following CLI requires a specific setup which will be covered below.

## Requirements

- Target CHANGELOG must follow conventional commits format.
- Releases must be fromatted like `chore(release): ...`
- Tags must hold the format of `vXX.XX.XX`
- The header of the CHANGELOG must be `# Changelog`

## Local Usage

```
$ cargo build --release
$ target/release/changie <args>
```

### CLI args
```
Usage: changie [OPTIONS] --org <ORG> --repo <REPO> --file-path <FILE_PATH> --target-version <TARGET_VERSION>

Options:
  -o, --org <ORG>
          Org name for the given repository
  -r, --repo <REPO>
          Name of the repository
  -f, --file-path <FILE_PATH>
          File path to the CHANGELOG
  -t, --target-version <TARGET_VERSION>
          Target version for the release. Format: vXX.XX.XX
  -s, --sha <SHA>
          Sha or branch to start commits at. Defaults to 'main' [default: main]
  -h, --help
          Print help
  -V, --version
          Print version                         Print version
```
