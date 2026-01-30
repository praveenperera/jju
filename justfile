fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

bacon:
    bacon

update:
    cargo update

fix:
    cargo clippy --fix --allow-dirty

release place="local":
    cargo xtask release {{place}}
