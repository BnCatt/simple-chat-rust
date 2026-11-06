#!/usr/bin/env python3
"""Package the mod: `target/simple_chat_rust-<version>.hnmod` and `target/simple_chat_rust-<version>.hmod`.

- The `.hnmod` holds `horizon-native.json` and `mixins/simple_chat_rust/` (its `.rs` and `.toml` files, the
  engine crate's included): the file today's launchers install (`horizon-native-mods/`) and build.
- The `.hmod` is the same native code plus the mod's content (`horizon-mod.toml` and `assets/`: its
  icons), one file for content and code as Horizon's docs/api/mod-files.md describes. Launchers up to
  Nightly 2026-11-05 still read only `.hnmod` files in `horizon-native-mods/`, so it is for the registry
  and later launchers. Without the content the mod draws its icons from the pixels compiled into it.

Both have fixed dates and modes, so the same sources always give the same bytes. `--install` also
copies the `.hnmod` into the launcher's native mods folder.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
if sys.platform == 'win32':
    INSTALLED = Path(os.environ['APPDATA']) / 'horizon' / 'horizon-native-mods'
elif sys.platform == 'darwin':
    INSTALLED = Path.home() / 'Library/Application Support/horizon/horizon-native-mods'
else:
    INSTALLED = Path.home() / '.local/share/horizon/horizon-native-mods'


def write(out: Path, files: dict):
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    return dict(file=str(out), files=len(files), size=out.stat().st_size,
                sha256=hashlib.sha256(out.read_bytes()).hexdigest())


def package():
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    bundle = ROOT / 'mixins' / 'simple_chat_rust'
    mixin = tomllib.loads((bundle / 'horizon-mixin.toml').read_text())
    mod = mixin['mod']
    content = tomllib.loads((ROOT / 'horizon-mod.toml').read_text())
    if mixin.get('compatibility', {}).get('horizon_revision') != compat['horizon_revision']:
        raise SystemExit('horizon-mixin.toml and compatibility.json pin different Horizon revisions')
    if (content['id'], content['version']) != (mod['id'], mod['version']):
        raise SystemExit('horizon-mod.toml and horizon-mixin.toml name different mods or versions')
    if (bundle / 'engine' / 'Cargo.lock').exists():
        raise SystemExit('remove mixins/simple_chat_rust/engine/Cargo.lock: a mod crate builds on Horizon\'s lock')
    # `hooks`: the native hook API it needs (client_packet 408, send_packet 409). The launcher
    # builds it on any Horizon with that API or later; `horizon` is the commit it was tested on.
    manifest = dict(schema=1, id=mod['id'], version=mod['version'], game=compat['game'],
                    horizon=compat['horizon_revision'], mixins=compat['horizon_mixins_revision'],
                    hooks=compat['hooks'])
    native = {'horizon-native.json': (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()}
    for path in sorted(bundle.rglob('*')):
        if path.is_file() and path.suffix in ('.rs', '.toml'):
            native[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    assets = {'horizon-mod.toml': (ROOT / 'horizon-mod.toml').read_bytes()}
    for path in sorted((ROOT / 'assets').rglob('*')):
        if path.is_file():
            assets[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    name = f"{mod['id']}-{mod['version']}"
    result = dict(
        hnmod=write(ROOT / 'target' / f'{name}.hnmod', native),
        hmod=write(ROOT / 'target' / f'{name}.hmod', {**native, **assets}),
    )
    if '--install' in sys.argv[1:]:
        INSTALLED.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / 'target' / f'{name}.hnmod', INSTALLED / f"{mod['id']}.hnmod")
        result['installed'] = str(INSTALLED / f"{mod['id']}.hnmod")
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    package()
