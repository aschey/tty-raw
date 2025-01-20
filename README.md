# tty-raw

Simple CLI program to print raw terminal inputs.
Probably only useful for developing input parsers or encoders.

```
Report raw terminal inputs

Usage: tty-raw [OPTIONS]

Options:
  -d, --disambiguate     keyboard enhancement flag - disambiguate escape codes
  -e, --all-escape       keyboard enhancement flag - report all keys as escape codes
  -a, --alternate-keys   keyboard enhancement flag - report alternate keys
  -t, --event-types      keyboard enhancement flag - report event types
  -k, --all-kitty        enable all kitty keyboard enhancements
  -b, --bracketed-paste  enable bracketed paste
  -m, --mouse            report mouse events
  -f, --focus            report focus change events
  -h, --help             Print help
  -V, --version          Print version
```
