# big-button

Just some interesting embedded Rust tutorial series I found floating around
the internet that I wanted to try.

Run me with `cargo run --release`

## Configurations

This project contains a `configs.json` file that contains various
configuration values for this program. The program will load these
values during the build process.

Currently the following configurations are supported:

```json
{
  "number_of_messages": "5"
}
```

## Secrets

Create and fill in a `secrets.json` file that contains sensitive
secret constants used by this project to run correctly.

Currently the following secrets are needed:

```json
{
  "super_secret_info": "Area51HasNoAliens"
}
```
