# Obsidian Dataview Switcher

For people who use obsidian, this allows you to switch a dataview query in a codeblock
into the format required by [obsidian-dataview-serializer](https://github.com/dsebastien/obsidian-dataview-serializer).

That is:

(This example purposefully uses only two backticks, to ensure all markdown renders show it)

```

  ``dataview
  LIST [[]]
    WHERE somecondition
  ``
```

becomes:

```
<!-- QueryToSerialize: LIST [[]] WHERE somecondition -->

```

### Usage

```
Usage: obsidian-dataview-switcher [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>
  -d, --dryrun <DRYRUN>  [possible values: true, false]
  -h, --help             Print help
  -V, --version          Print version


```
