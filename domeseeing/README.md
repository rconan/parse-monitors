# Dome Seeing

Ray tracing through the GMT CFD dome seeing turbulence volume and PSSn estimation.

# Environment variables

```bash
export GMT_MODES_PATH=/path/to/ceo-mirror-modes/
```

# Usage

```shell
cargo r -r --bin domeseeing -- --help
```

## Example on AWS

```shell
export GMT_MODES_PATH=~/CEO/gmtMirrors
cargo r -r --bin domeseeing -- --case /home/ubuntu/cfd/CASES/<cfd-case>
```
