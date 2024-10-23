# aftermath
From a `projects.toml`, replace strings and commit changes.

## `project.toml`
```toml
[[projects]]
url = "https://github.com/wcampbell0x2a/librarium"
name = "librarium"
replace_prefix = "runs-on:"
yaml_path = ".github/workflows/main.yml"
```
## Run
```
$ aftermath projects.toml wcampbell wcampbell1995@gmail.com rust:0.1.2 --root-dir tmp
````
