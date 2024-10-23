# aftermath
From a `projects.toml`, replace every project with new ci yml node and commit changes.

## `project.toml`
```toml
[[projects]]
url = "https://github.com/wcampbell0x2a/librarium"
name = "librarium"
replace_prefix = "runs-on:"
yaml_path = ".github/workflows/main.yml"
```
## Run
Replace the `runs-on:` with latest `rust:0.1.2`.
```
$ aftermath projects.toml wcampbell wcampbell1995@gmail.com rust:0.1.2 --root-dir tmp
````
