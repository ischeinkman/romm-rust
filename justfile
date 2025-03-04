
set dotenv-load := true

default:
    @just --list

[group('package')]
package-miyoo: build-miyoo 
    if [ ! -d .build ]; then mkdir .build; fi
    cp -r assets/miyoo-mini .build 
    cp target/arm-unknown-linux-musleabihf/release/syncer-daemon .build/miyoo-mini 
    cp target/arm-unknown-linux-musleabihf/release/syncer-ui-miyoo .build/miyoo-mini
    sed -i "s%.*\$ROMM_URL.*%url = \"$ROMM_URL\"%" .build/miyoo-mini/config.toml
    sed -i "s%.*\$ROMM_API_KEY.*%api-key = \"$ROMM_API_KEY\"%" .build/miyoo-mini/config.toml

alias pkg := package-miyoo
alias pkg-miyoo := package-miyoo

[group('package')]
package-miyoo-clean: 
    rm -rf .build/miyoo-mini 

alias clean-miyoo := package-miyoo-clean

[group('build')]
build-miyoo:
    cross build --target arm-unknown-linux-musleabihf --release

test:
    cargo test 
alias t := test 

clean:
    cargo clean 
    cross clean 
    rm -rf .build