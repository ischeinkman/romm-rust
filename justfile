
set dotenv-load := true

default:
    @just --list

[group('package')]
package-miyoo: build-miyoo 
    #!/bin/bash 

    if [ ! -d .build ]; then mkdir .build; fi
    cp -r assets/miyoo-mini .build 
    cp target/arm-unknown-linux-musleabihf/release/syncer-daemon .build/miyoo-mini 
    cp target/arm-unknown-linux-musleabihf/release/syncer-ui-miyoo .build/miyoo-mini
    if [ -z "$ROMM_URL" ]; 
        then echo "WARNING: $ROMM_URL not set; add to the config manually." >&2; 
        else sed -i "s%.*\$ROMM_URL.*%url = \"$ROMM_URL\"%" .build/miyoo-mini/config.toml;
    fi 
    if [ -z "$ROMM_API_KEY" ]; 
        then echo "WARNING: $ROMM_API_KEY not set; add to the config manually." >&2; 
        else sed -i "s%.*\$ROMM_API_KEY.*%api_key = \"$ROMM_API_KEY\"%" .build/miyoo-mini/config.toml
    fi 
    echo "Finished building the package under /.build/miyoo-mini"

alias pkg := package-miyoo
alias pkg-miyoo := package-miyoo

[group('package')]
package-miyoo-clean: 
    rm -rf .build/miyoo-mini 

alias clean-miyoo := package-miyoo-clean

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