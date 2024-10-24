# aftermath
Are you tired of updating your build image tags in every project manually? Enter `aftermath`, where you can do
this with automation! `aftermath` will find-and-replace, commit, test, and push the changes.

## Example
Replace the `runs-on:` with latest `rust:0.1.2`.
```toml
[[projects]]
url = "https://github.com/wcampbell0x2a/librarium"
name = "librarium"
replace_prefix = "runs-on:"
yaml_path = ".github/workflows/main.yml"
```
```
$ aftermath projects.toml wcampbell wcampbell1995@gmail.com rust:0.1.2 --root-dir tmp
````
