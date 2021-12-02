from itertools import tee
import pandas as pd


def pairwise(iterable):
    # pairwise('ABCDEFG') --> AB BC CD DE EF FG
    a, b = tee(iterable)
    next(b, None)
    return zip(a, b)


example = """
199
200
208
210
200
207
240
269
260
263
"""

example = map(int, example.strip().split('\n'))
rolling = pd.Series(example).rolling(3).sum()

answer = len(list(None for a, b in pairwise(rolling) if a < b))
print(answer)
