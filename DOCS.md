# Link Shortener API Documentation

---

## Table of contents

- [Configuration](#configuration)
  - [Overview](#overview)
  - [Configuration keys](#configuration-keys)

---

## Configuration

### Overview

Configuration is stored inside of `Config.toml` file. It follows the same rules as Rocket's configuration (`debug` and `release` profiles, `default` profile for values applied to both, `global` for overrides, etc.)

### Configuration keys

| Key | Description |
| :---: | :---:
| `database_url` | Specifies URL used by diesel/r2d2 for connection to MySQL database |

---

### [Mikut](https://mikut.dev) 2020-2022