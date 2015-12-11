# Legit

Legit is a tool for generating git commits with a custom commit hash prefix, like "000000".

As an example, take a look at a few of the commits in this repo.

### Usage

```shell
cd ~/Projects/my-repo
git add README.md
legit ./ -m "Add a README" -p "000000"
```

### Compiling

```shell
brew tap nerdrew/tap
brew install rust-nightly
brew install openssl
cd cloned_directory
export OPENSSL_INCLUDE_DIR=$(brew prefix openssl)/include/
cargo build
```

### Warning

__LEGIT WILL REVERT ANY UNSTAGED MODIFIED FILES IN YOUR REPOSITORY.  YOU WILL LOSE DATA IF YOU DO NOT REMEMBER THIS.__
