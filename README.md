# anti-spam-matrix

This is a simple Matrix spam banning bot.

Its logic is currently straightforward:

If a user triggers the specified keyword in a certain number of consecutive messages, they will be banned.

If a user triggers the keyword in numerous (the spam_limit) consecutive messages in different groups, they will also be banned.

The bot will ban the spammer in all rooms where it has permissions.

## Usage

Currently we support two authurization methods, `sso` and `password`

```toml
[auth]
type = "password"
password = "VeryHardPassword"
```

> Note: In SSO login, username part of the userid will be ignored.

```toml
[auth]
type = "sso_login"
```
