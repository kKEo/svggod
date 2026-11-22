# Exploratory testing by hand

This page is a guided half-hour session: it uses the commands and fixtures that ship with SVGGod to look for problems that the automated tests do not cover.

Run the commands from the repository root with a release binary (`cargo build --release`, binary at `target/release/svggod`). Write outputs to a scratch directory, not into the repository.

```bash
alias svggod=./target/release/svggod
mkdir -p /tmp/svggod-explore
```

## 1. Look at a deliberately broken file

`tests/fixtures/broken.svg` is a hand-made file with many faults. Validate it:

```bash
svggod validate tests/fixtures/broken.svg --no-config ; echo "exit $?"
```

On version 1.0.0 you should see `"ok":false`, the summary `errors 7, warnings 11, info 2, fixable 15`, and exit code `1`. The findings cover 18 different rules, from `title-missing` to `foreign-object`. Anything that differs from this is worth a note.

Try the controls:

```bash
svggod validate tests/fixtures/broken.svg --no-config --rules low-contrast,duplicate-id
svggod validate tests/fixtures/broken.svg --no-config --skip desc-missing --fail-on warning
svggod validate tests/fixtures/broken.svg --profile minimal
svggod validate tests/fixtures/broken.svg --max-findings 3     # summary counts stay complete
```

## 2. See what the findings point at

Findings that sit on a drawn element carry a bounding box. Draw them:

```bash
svggod preview tests/fixtures/broken.svg -o /tmp/svggod-explore/broken.png --annotate
svggod preview tests/fixtures/broken.svg -o /tmp/svggod-explore/crop.png --crop '#a' --background dark
```

Open the PNGs. With `--annotate`, each finding's box is outlined with its rule id. Check that the outline sits on the element the message talks about. Findings marked `"approximate": true` in the JSON come from geometry measured on a file SVGGod did not draw, so small offsets are expected there.

## 3. Ask the scene questions

`inspect` reports what is on the page:

```bash
svggod inspect tests/fixtures/orders.svg --select '#node-api' --fields id,tag,bbox,text
svggod inspect tests/fixtures/orders.svg --depth 1
```

On version 1.0.0 the clean `orders.svg` has a `viewBox` of `0 0 720 240` and 17 elements, and `#node-api` returns three: the group, its `rect` and its `text`. Check the boxes against the image from `svggod preview`.

## 4. Fix it, then check what the fixer did

```bash
svggod fix tests/fixtures/broken.svg --dry-run          # reports the diff, writes nothing
svggod fix tests/fixtures/broken.svg -o /tmp/svggod-explore/fixed.svg
svggod validate /tmp/svggod-explore/fixed.svg --no-config
svggod fix /tmp/svggod-explore/fixed.svg --dry-run      # must be an empty diff (R9)
```

After the safe fixes, the dry run on version 1.0.0 leaves `errors 6, warnings 5, fixable 6`. The remaining findings need a human or `--unsafe`. Then try `--unsafe` and look at the picture again: unsafe fixes move or recolour content, so check that nothing now overlaps.

## 5. Break the inputs on purpose

| Try | Expect |
|---|---|
| `svggod validate nope.svg` | exit code 3 and `"code":"file-not-found"` |
| `svggod validate tests/fixtures/orders.svg --bogus` | exit code 2 and `"code":"unknown-flag"` |
| `svggod frobnicate` | exit code 2 |
| a spec with a misspelt key, such as `lable:` instead of `label:` | a "did you mean" suggestion with the path |
| a spec with two nodes sharing an id | `duplicate-id` error naming both |
| a 40-node spec in `flow` direction `right` | either a clean render or a `too-wide` error with a hint |
| an SVG with a UTF-8 label, an emoji, or a very long label | no crash; check the picture |
| `svggod render spec.yaml -o -` and `svggod validate -` | stdout and stdin forms work |
| rendering the same spec twice and running `diff` | no difference (R8) |

## 6. Edit with comments in the spec

Copy `tests/fixtures/orders.yaml`, add `# my comment` lines, then edit through svggod:

```bash
cp tests/fixtures/orders.yaml /tmp/svggod-explore/o.yaml
# add a comment to the file, then:
svggod edit /tmp/svggod-explore/o.yaml --set '#api.label=Orders API' --in-place
diff tests/fixtures/orders.yaml /tmp/svggod-explore/o.yaml
```

The comments should survive, and the JSON envelope should report `"formatting": "preserved"`. Removing a node always rewrites the file in canonical form (`"formatting": "rewritten"`); that is expected. The test `spec_edits_keep_yaml_comments` covers the basic case, so spend your time on odd layouts: flow-style lists, anchors, multi-line strings, Windows line endings.

## 7. Drive the MCP server

```bash
mkdir -p /tmp/svggod-explore/root && cp tests/fixtures/orders.yaml /tmp/svggod-explore/root/
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | svggod mcp --root-path /tmp/svggod-explore/root
```

You should get one JSON line listing eight tools. Then try a call whose path escapes the root (`../x.svg`): it must be refused.

## Write down what you find

For each problem, note the command, the input file, the expected and the actual result, and the version (`svggod --version`). Turn every real bug into a test with [Adding tests and the pull-request checklist](adding-tests.md).
