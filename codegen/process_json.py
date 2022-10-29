import json
import os
from typing import Callable, Any

def process(filename: str, filter_condition: Callable[[tuple[str, dict[str, Any]]], bool], out_fields: list[str]):
    with open(filename, 'r') as f:
        j = json.load(f)
        
        up = j['unprefixed']
        p = j['cbprefixed']
        ret: dict[str, set[tuple[str, Any]]] = {'UNPREFIXED': set(), 'PREFIXED': set()}
        
        for ops in map(lambda x: (x[0], (x[1][key] for key in out_fields)), filter(filter_condition, up.items())):
            ret['UNPREFIXED'].add(ops)
        for ops in map(lambda x: (x[0], (x[1][key] for key in out_fields)), filter(filter_condition, p.items())):
            ret['PREFIXED'].add(ops) 
        return ret

def process2(filename: str, filter_condition: Callable[[tuple[str, dict[str, Any]]], bool], out_fields: list[str]):
    with open(filename, 'r') as f:
        j = json.load(f)
        
        up = j['unprefixed']
        p = j['cbprefixed']
        ret: dict[str, Any] = {'UNPREFIXED': [], 'PREFIXED': []}
        
        for ops in map(lambda x: (x[0], [x[1][key] for key in out_fields]), filter(filter_condition, up.items())):
            ret['UNPREFIXED'].append(ops)
        for ops in map(lambda x: (x[0], [x[1][key] for key in out_fields]), filter(filter_condition, p.items())):
            ret['PREFIXED'].append(ops) 
        return ret
def main():
    cur_dir = os.path.dirname(__file__)
    # fn = os.path.join(cur_dir, 'Opcodes.json')
    fn = os.path.join(cur_dir, 'op_raw.json')
    mnemonics = process2(fn, lambda x: x[1]['group'] == 'x8/lsm', ['addr'])
    # mnemonics = process2(fn, lambda x: x[1]['group'] == 'x8/alu', ['operand1'])
    for (t, i) in mnemonics.items():
        print(t) 
        # i = list(map(lambda x: x[0], i))
        print(i)

if __name__ == '__main__':
    main()