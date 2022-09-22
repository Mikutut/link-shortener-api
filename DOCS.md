# Link Shortener API Documentation

---

## Table of contents

- [Configuration](#configuration)
  - [Overview](#overview)
  - [Configuration keys](#configuration-keys)
- [Errors](#errors)
  - [Overview](#overview-1)
  - [Error types](#error-types)

---

## Configuration

### Overview

Configuration is stored inside of `Config.toml` file. It follows the same rules as Rocket's configuration (`debug` and `release` profiles, `default` profile for values applied to both, `global` for overrides, etc.)

### Configuration keys

| Key | Description | Default |
| :---: | :---: | :---: |
| `database_url` | Specifies URL used by diesel/r2d2 for connection to MySQL database | `mysql://root:root@localhost:3306/link_shortener` |
| `max_requests` | Specifies how many requests client can make in time window | `100` |
| `max_requests_time_window` | Specifies the time window (in seconds) for rate limiter Defaults to | `10800` (3 hours) |
| `base_url` | Specifies base URL returned when creating/editing link (link ID will be appended to it) | `http://localhost` |
| `max_auto_id_length` | Specifies how long auto-generated link IDs can be | `6` |
| `max_id_length` | Specifies how long link IDs provided by user can be (API does **NOT** check if this value is equal or not to link ID column in database!) | `255` |

---

## Errors

### Overview

Sometimes, API may return an error. In that case, response will include appropriate an error type and an error message, the latter of which should clarify the source of the former. Proper error handling should consist of:

1. Inspecting returned HTTP code
2. Inspecting returned error type
3. (Optionally) inspecting returned error message (may be harder to do, since error messages are not meant to be parsed but rather read by API user directly)

### Error types

| Type | Description |
| :---: | :---:
| `ValidationError` | Server could not parse request data correctly. |
| `DatabaseError` | Server could not communicate with database properly. |
| `DuplicateIdError` | A link with the same ID as provided in request data already exists in database. |
| `InvalidControlKeyError` | Provided control key for a link is invalid. |
| `RateLimitedError` | Integrated rate limiter has detected too many requests in too short period of time and so your requests have been blocked. Please wait provided amount of seconds before sending another request. |
| `LinkNotFoundError` | Link with provided ID has not been found in database. |
| `ControlKeyHashGenerationError` | Server could not generate bcrypt hash of new control key. |
| `ControlKeyHashVerificationError` | Server could not verify bcrypt hash acquired from database. |
| `GetLinksError` | Loosely specified error regarding getting list of links. Refer to error message for more information. |
| `AccessLinkError` | Loosely specified error regarding accessing link. Refer to error message for more information. |
| `AddLinkError` | Loosely specified error regarding adding link. Refer to error message for more information. |
| `EditLinkError` | Loosely specified error regarding editing link. Refer to error message for more information. |
| `DeleteLinkError` | Loosely specified error regarding deleting link. Refer to error message for more information. |
| `UndefinedError` | Server has thrown an error that did not fit into any of the aforementioned types. Refer to error message for more information. |

---

### [Mikut](https://mikut.dev) 2020-2022
