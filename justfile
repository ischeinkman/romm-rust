
set dotenv-load := true

default:
    @just --list

package-miyoo-zip: package-miyoo 
    #!/bin/bash 
    zip -j .build/sync-saver-miyoo.zip .build/sync-saver/*

alias release-miyoo := package-miyoo-zip

[group('package')]
package-miyoo: package-miyoo-clean build-miyoo 
    #!/bin/bash 
    if [ ! -d .build ]; then mkdir .build; fi
    cp -r assets/miyoo-mini .build/sync-saver
    cp target/arm-unknown-linux-musleabihf/release/syncer-daemon .build/sync-saver
    cp target/arm-unknown-linux-musleabihf/release/syncer-ui-miyoo .build/sync-saver
    if [ -z "$ROMM_URL" ]; 
        then echo "WARNING: $ROMM_URL not set; add to the config manually." >&2; 
        else sed -i "s%.*\$ROMM_URL.*%url = \"$ROMM_URL\"%" .build/sync-saver/config.toml;
    fi 
    if [ -z "$ROMM_API_KEY" ]; 
        then echo "WARNING: $ROMM_API_KEY not set; add to the config manually." >&2; 
        else sed -i "s%.*\$ROMM_API_KEY.*%api_key = \"$ROMM_API_KEY\"%" .build/sync-saver/config.toml
    fi 
    echo "Finished building the package under /.build/sync-saver"

alias pkg := package-miyoo
alias pkg-miyoo := package-miyoo

[group('package')]
package-miyoo-clean: 
    rm -rf .build/sync-saver

[group('build')]
build-miyoo:
    cross build --target arm-unknown-linux-musleabihf --release

fmt: 
    cargo fmt

test:
    cargo test 
alias t := test 

clean:
    cargo clean 
    cross clean 
    rm -rf .build