# Pump19
## Configuration
Pump19 is configured exclusively through **environment variables** as suggested by [The Twelve Factor App](https://12factor.net/).

This is an example of a supported `.env` file:
```bash
# a regular expression for trigger characters
PUMP19_TRIGGERS = "^[?!💩]"

# the IRC settings
PUMP19_IRC_NICKNAME = "pump19"
PUMP19_IRC_USERNAME = "pump19"
PUMP19_IRC_PASSWORD = "oauth:53cr3t04uthp4ssw0rdfr0mtw1tch"
PUMP19_IRC_CHANNELS = "#loadingreadyrun,#desertbus"

# codefall database and channels to use for the announce feature
PUMP19_CODEFALL_DATABASE = "postgresql://user@host:port/db"
PUMP19_CODEFALL_CHANNELS = "#desertbus"

# URLs to print for certain commands
PUMP19_URL_HELP = "https://pump19.eu/commands"
PUMP19_URL_BINGO = "https://pump19.eu/bingo"
PUMP19_URL_CODEFALL = "https://pump19.eu/codefall"

# logging configuration
RUST_LOG = "debug"
```
