# Legit

Legit is a tool for generate git commits with a custom commit hash prefix, for example "000000"

### Usage

```
cd ~/Projects/my-repo
git add README.md
legit ./ -m "Add a README" -p "badc0de"
```

### Warning

__LEGIT WILL REVERT ANY UNSTAGED MODIFIED FILES IN YOUR REPOSITORY.  YOU WILL LOSE DATA IF YOU DO NOT REMEMBER THIS.__
