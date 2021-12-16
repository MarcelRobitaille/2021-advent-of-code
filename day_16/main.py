"""
This was a speedrun attempt (part one only). It is horrible code.
"""

import sys
from functools import reduce
from operator import mul, add
import math
import heapq
import collections
from itertools import tee
from collections import Counter, defaultdict, deque
from colorama import Fore, Style

import pandas as pd
import numpy as np


def parse(line):
    for c in line:
        yield {
            '0': '0000',
            '1': '0001',
            '2': '0010',
            '3': '0011',
            '4': '0100',
            '5': '0101',
            '6': '0110',
            '7': '0111',
            '8': '1000',
            '9': '1001',
            'A': '1010',
            'B': '1011',
            'C': '1100',
            'D': '1101',
            'E': '1110',
            'F': '1111',
        }[c]

ans = 0

def parse_packet(line):
    print(line)
    global ans
    ver = int(line[:3], 2)
    print('ver', ver)
    ans += ver
    typ = int(line[3:6], 2)
    print('typ', typ)
    line = line[6:]
    if typ == 4:
        acc = ''
        while line:
            n = line[1:5]
            acc += (n)
            shouldbreak = line[0]
            line = line[5:]

            if shouldbreak == '0':
                return int(acc, 2), line
    # Operator
    else:
        tot = []
        type_id = line[0]
        print('type_id', type_id, line)
        line = line[1:]
        if type_id == '0':
            length = int(line[:15], 2)
            print('length', length)
            line = line[15:]
            i = 0
            while i <= length - 1:
                prev_len = len(line)
                res, line = parse_packet(line)
                tot.append(res)
                new_len = len(line)
                i += prev_len - new_len
                print('eaten', prev_len - new_len)
                print(i, length)
                print('res', res)

            print('line', line)
            print('length', length)
        elif type_id == '1':
            subs = int(line[:11], 2)
            print('subs', subs)
            line = line[11:]
            for i in range(subs):
                res, line = parse_packet(line)
                tot.append(res)
                print('res', i, res)
        else:
            raise Exception()

        f = {
            0: add,
            1: mul,
            2: min,
            3: max,
            5: lambda x, y: 1 if x > y else 0,
            6: lambda x, y: 1 if x < y else 0,
            7: lambda x, y: 1 if x == y else 0,
        }[typ]
        return reduce(f, tot), line

for line in test_cases.strip().split('\n'):
    print(parse_packet(bin(int(line, 16))[2:]))

# %% test

with open('./example.txt', 'r') as f:
    test_cases = f.read()

# %% real

with open('./real_input.txt', 'r') as f:
    test_cases = f.read()
