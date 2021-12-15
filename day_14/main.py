"""
This was a speedrun attempt (part one only). It is horrible code.
I got 12:46 (rank 1448).
"""

from itertools import tee
import sys
import pandas as pd
import numpy as np
import math
import collections


def pairwise(iterable):
    # pairwise('ABCDEFG') --> AB BC CD DE EF FG
    a, b = tee(iterable)
    next(b, None)
    return zip(a, b)

template, pair_insertions = test_cases.split('\n\n')


res = {}
for line in pair_insertions.strip().split('\n'):
    res[line[0:2]] = line[-1]


res
# %%

counts = collections.defaultdict(int)
for (a, b) in pairwise(template):
    counts[a + b] += 1

print(counts.items())
for _ in range(40):
    new_counts = collections.defaultdict(int)
    for key, count in counts.items():
        new_counts[key[0] + res[key]] += count
        new_counts[res[key] + key[1]] += count
    counts = new_counts
    # print(counts.items())
# for _ in range(40):
#     new = thing
#     rolling = 0
#     for i, (a, b) in enumerate(pairwise(thing)):
#         for match, insert in res:
#             if a + b == match:
#                 j = i + 1 + rolling
#                 rolling += 1
#                 # print(match) 
#                 # print(new, new[:j] + insert + new[j:])
#                 new = new[:j] + insert + new[j:]
#                 break
#     thing = new
# print(thing)
# print(collections.Counter(thing))
# %%
elem_counts = collections.defaultdict(int)

for key, count in counts.items():
    for letter in key:
        elem_counts[letter] += count / 2
elem_counts[template[0]] += 0.5
elem_counts[template[-1]] += 0.5

# elem_counts
max(elem_counts.values()) - min(elem_counts.values())

# %% test

with open('./test_case.txt', 'r') as f:
    test_cases = f.read()

# %% real

with open('./real_input.txt', 'r') as f:
    test_cases = f.read()
