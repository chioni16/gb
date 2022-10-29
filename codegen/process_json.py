import json
import os
from typing import Callable, Any

def process(filename: str, filter_condition: Callable[[tuple[str, dict[str, Any]]], bool], out_fields: list[str]):
    with open(filename, 'r') as f:
        j = json.load(f)
        
        up = j['unprefixed']
        p = j['cbprefixed']
        ret: dict[str, set[tuple[str, Any]]] = {'UNPREFIXED': set(), 'PREFIXED': set()}
        # for ops in map(lambda x: x[0], filter(lambda x: x[1]['bytes'] > 1, up.items())):
        #     mnemonics['up'].add(ops)
        # for ops in map(lambda x: x[0], filter(lambda x: x[1]['bytes'] > 2, p.items())):
        #     mnemonics['p'].add(ops)
        for ops in map(lambda x: (x[0], (x[1][key] for key in out_fields)), filter(filter_condition, up.items())):
            ret['UNPREFIXED'].add(ops)
        for ops in map(lambda x: (x[0], (x[1][key] for key in out_fields)), filter(filter_condition, p.items())):
            ret['PREFIXED'].add(ops) 
        return ret
    
def main():
    cur_dir = os.path.dirname(__file__)
    # fn = os.path.join(cur_dir, 'Opcodes.json')
    fn = os.path.join(cur_dir, 'Opcodes.json')
    mnemonics = process(fn, lambda x: True, ['mnemonic'])
    for (t, i) in mnemonics.items():
        print(t)
        s: set[str] = set()
        for j in i:
            for v in j[1]:
                s.add(v) 
        print(s)

if __name__ == '__main__':
    main()