import sys
from itertools import tee
import pandas as pd


def pairwise(iterable):
    # pairwise('ABCDEFG') --> AB BC CD DE EF FG
    a, b = tee(iterable)
    next(b, None)
    return zip(a, b)


def parse_input():
    return [int(l.rstrip()) for l in sys.stdin]


def part_one(values):
    return len(list(None for a, b in pairwise(values) if a < b))


def part_two(values):
    rolling = pd.Series(values).rolling(3).sum()
    return len(list(None for a, b in pairwise(rolling) if a < b))


def main():
    try:
        command = sys.argv[1]
        fn = {
            'part-one': part_one,
            'part-two': part_two,
        }[command]
    except IndexError:
        print('Please specify `part-one\' or `part-two\' as the first argument.', file=sys.stderr)
        sys.exit(1)
    except KeyError:
        print(f'Invalid command `{command}\'. Expected `part-one\' or `part-two\'.', file=sys.stderr)
        sys.exit(1)

    print(fn(parse_input()))


if __name__ == '__main__':
    main()
