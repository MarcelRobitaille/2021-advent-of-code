"""
This was a speedrun attempt (part one only). It is horrible code.
"""

import sys
import math
import heapq
import collections
from itertools import tee
from collections import Counter, defaultdict, deque
from colorama import Fore, Style

import pandas as pd
import numpy as np


def pairwise(iterable):
    # pairwise('ABCDEFG') --> AB BC CD DE EF FG
    a, b = tee(iterable)
    next(b, None)
    return zip(a, b)



grid = []
for line in test_cases.strip().split('\n'):
    grid.append([int(x) for x in line])
grid

X_EXTENT = len(grid[0])
Y_EXTENT = len(grid)

# %% Grow map

weights = {}
for x in range(X_EXTENT * 5):
    for y in range(Y_EXTENT * 5):
        region_x = x // X_EXTENT
        region_y = y // Y_EXTENT
        weights[(x, y)] = (grid[x % X_EXTENT][y % Y_EXTENT] + region_x + region_y - 1) % 9 + 1
weights

for x in range(X_EXTENT * 5):
    for y in range(Y_EXTENT * 5):
        print(f'{weights[(x, y)]}', end='')
    print()

X_EXTENT *= 5
Y_EXTENT *= 5
# %% Dijkstra


start = (0, 0)
target = (X_EXTENT - 1, Y_EXTENT - 1)

q = []
# heapq.heappush(q, (grid[0][0], start))
q.append(start)

dist = defaultdict(lambda: np.inf)
prev = defaultdict()

dist[start] = weights[start]

seen = set()

while q:
    # weight, u = heapq.heappop(q)
    print(q)
    u = min(q, key=dist.get)
    q.remove(u)

    if u in seen:
        continue

    seen.add(u)

    x, y = u
    neighbours = [v for v, t in {
        (x - 1, y): x > 0,
        (x + 1, y): x < X_EXTENT - 1,
        (x, y - 1): y > 0,
        (x, y + 1): y < Y_EXTENT - 1,
    }.items() if t]
    for v in neighbours:
        if v in seen:
            continue
        alt = dist[u] + weights[u]
        # assert w != target
        if alt < dist[v]:
            dist[v] = alt
            prev[v] = u
        # heapq.heappush(q, (weights[v], v))
        q.append(v)
        # if w == target:
        #     break


w = target
path = []
weight = 0
while w != start:
    print(w)
    path.append(w)
    weight += weights[w]
    w = prev[w]
for x in range(X_EXTENT):
    for y in range(Y_EXTENT):
        if (x, y) in path:
            print(f'{Fore.RED}{weights[(x,y)]}{Style.RESET_ALL}', end='')
        else:
            print(f'{weights[(x, y)]}', end='')
    print()



# %% test

with open('./example.txt', 'r') as f:
    test_cases = f.read()

# %% real

with open('./test.txt', 'r') as f:
    test_cases = f.read()
