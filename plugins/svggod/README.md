# svggod plugin for Claude Code

Three skills that let Claude work with [SVGGod](https://github.com/kKEo/svggod) without
rediscovering the tool each time.

| Skill | Use it to |
|---|---|
| `svggod:install` | Put the `svggod` binary on this machine and confirm it works |
| `svggod:draw` | Make or change a diagram: write a spec, render it, look at the result |
| `svggod:check` | Check, repair or edit SVG files that already exist, from any tool |

`draw` and `check` load on their own when a request matches. `install` runs when you ask for it,
or when one of the other two finds that `svggod` is missing.

## Add it

From a checkout of this repository:

```sh
claude plugin marketplace add /path/to/svggod
claude plugin install svggod@svggod
```

Or, inside Claude Code, `/plugin marketplace add /path/to/svggod` and then
`/plugin install svggod@svggod`. The GitHub form works too:
`/plugin marketplace add kKEo/svggod`.

After changing a skill, run `claude plugin marketplace update svggod` and restart the session.

## Layout

```
skills/install/SKILL.md
skills/draw/SKILL.md      + references/ (spec, explaining, markdown-pages, themes, errors)
skills/check/SKILL.md     + references/ (rules-and-config, editing)
```

Themes, diagrams in Markdown pages and step-by-step walk-throughs are reference files of `draw`
rather than skills of their own: they only come up while drawing, and Claude reads them when the
task needs them.
