# Simple Romm Syncer

An **unofficial, third party, not-officially-supported, and above all IN
PROGRESS** tool for syncing saves between the local device and a remote ROMM
server.

## Screenshots

### Miyoo Mini

![Homapage Screenshot](/assets/screenshots/homepage.bmp)

![Save List Screenshot](/assets/screenshots/save-list.bmp)

## Usage

### Miyoo Mini

To install the syncer on your Miyoo Mini: 

1. Generate your Authorization header. At the time of writing the only way to do
   this consistently is to base64 your `$USERNAME:$PASSWORD` string and then put
   the word `Basic` in front; for example, if your username is `admin` and your
   password is `admin`, your Authorization header would be `Basic
   YWRtaW46YWRtaW4K`.
2. Download the `sync-saver-miyoo.zip` file from the [releases
   page](https://github.com/ischeinkman/romm-syncer/releases).
3. Extract the zip file under `/mnt/SDCARD/App/Romm_Save_Syncer` (or whatever
   you want to call it). 
4. Modify `/mnt/SDCARD/App/Romm_Save_Syncer/config.toml` using a text editor to
   set the `system.url` to your Romm server's URl and `system.api-key` to the
   key generated in step 1. 
5. On your Miyoo Mini, go into `Apps`. You should see a new application called `Romm Save Syncer` in the list. Open it. 
6. From here you can:
   * Start & stop the syncer daemon
   * Install a shim wrapper so the syncer daemon starts at every boot instead of
     needing to be restarted manually whenever the Miyoo Mini reboots, or
     uninstall the shim so it doesn't
   * Change how often the sync status is polled and whether or not a filesystem
     notification changes a save
   * Enable & disable syncing for specific saves 

Log files for the configuration UI, the daemon, and the shim are all saved
alongside the application file under `/mnt/SDCARD/App/Romm_Save_Syncer` as a
variety of `.out` and `.err` files. 

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