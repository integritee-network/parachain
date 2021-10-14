#!/usr/bin/env python3
import json
import os
import subprocess

SPECS = [
    {
        "chain_id": "shell-kusama",
        "para_id": 2015,
    },
    {
        "chain_id": "integritee-kusama",
        "para_id": 2015,
    }
]
COLLATOR = "target/release/integritee-collator"
RES_DIR = "polkadot-parachains/res"


def main():
    for s in SPECS:
        chain_spec = s["chain_id"]

        ret = subprocess.call(
            f'scripts/dump_wasm_state_and_spec.sh {chain_spec}-fresh {s["para_id"]} {COLLATOR} {RES_DIR}',
            stdout=subprocess.PIPE,
            shell=True
        )

        print(ret)

        orig_file = f'{RES_DIR}/{chain_spec}.json'
        new_file_base = f'{RES_DIR}/{chain_spec}-fresh'
        with open(orig_file, 'r+') as spec_orig_file:
            orig_json = json.load(spec_orig_file)

            # migrate old values to new spec

            with open(f'{new_file_base}-raw.json', 'r+') as spec_new_file:
                new_json = json.load(spec_new_file)

                new_json["bootNodes"] = orig_json["bootNodes"]

                # go to beginning of the file to overwrite
                spec_orig_file.seek(0)
                json.dump(new_json, spec_orig_file, indent=2)
                spec_orig_file.truncate()

        os.remove(f'{new_file_base}.json')
        os.remove(f'{new_file_base}-raw.json')
        os.remove(f'{new_file_base}.state')
        os.remove(f'{new_file_base}.wasm')






if __name__ == '__main__':
    main()
