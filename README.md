# Simple Romm Syncer

An **unofficial, third party, not-officially-supported, and above all IN
PROGRESS** tool for syncing saves between the local device and a remote ROMM
server.


## Usage

### Miyoo Mini


## Building

### Miyoo Mini

Build requirements:
* `cargo`
* [`cross`](https://github.com/cross-rs/cross) (for cross-compilation)
* [`just`](https://github.com/casey/just) (for command running)

Build steps:

1. Generate your Authorization header. At the time of writing the only way to do
   this consistently is to base64 your `$USERNAME:$PASSWORD` string and then put
   the word `Basic` in front; for example, if your username is `admin` and your
   password is `admin`, your Authorization header would be `Basic
   YWRtaW46YWRtaW4K`.
2. Put your Romm URL and API key into the environment variables `$ROMM_URL` and
   `$ROMM_API_KEY`, respectively. 
3. Run `just pkg`.

This will generate the full `sync-saver` app directory under `.build/`,
including a `config.toml` file with your API key and server URL already
populated. Just transfer the app directory to `SDCARD/Apps` and you should be
set.

## Components

* `syncer-daemon` -- A program that sits in the background syncing saves between
  the device and external Romm server periodically based on the configured
  parameters.
* `syncer-ui-miyoo` -- The UI for configuring the save syncing daemon on the
  Miyoo Mini.
* `syncer-model` -- The base communication code used to keep the daemon & all UIs in sync.
* `romm-api` -- A crate containing the structs needed to interact with Romm's
  REST API.

## Progress

- [x] Romm integration
- [x] Syncing between local saves & Romm server
- [x] Continous syncing in the background
- [x] Miyoo Mini UI
- [ ] Desktop UI