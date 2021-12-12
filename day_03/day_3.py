import sys
from collections import Counter

import pandas as pd


def get_rate(diagnostic_report, commonness_selector):
    def mapper(column):
        counter = Counter(column)
        return commonness_selector(counter, key=counter.get)
    return int(''.join(diagnostic_report.apply(mapper)), 2)


def part_one(diagnostic_report):
    gamma_rate = get_rate(
        diagnostic_report=diagnostic_report,
        commonness_selector=max,
    )
    epsilon_rate = get_rate(
        diagnostic_report=diagnostic_report,
        commonness_selector=min,
    )
    return gamma_rate * epsilon_rate


def get_rating(diagnostic_report, tie_breaker, commonness_selector):
    def recurse(diagnostic_report, bit_position):
        if len(diagnostic_report) == 1:
            return int(''.join(diagnostic_report.iloc[0]), 2)

        counter = Counter(diagnostic_report[bit_position])
        mode = tie_breaker \
            if len(set(counter.values())) == 1 else \
            commonness_selector(counter, key=counter.get)

        return recurse(
            diagnostic_report=diagnostic_report[diagnostic_report[bit_position] == mode],
            bit_position=bit_position + 1,
        )

    return recurse(diagnostic_report=diagnostic_report.copy(), bit_position=0)


def part_two(diagnostic_report):
    oxygen_rating = get_rating(
        diagnostic_report=diagnostic_report,
        tie_breaker='1',
        commonness_selector=max,
    )
    co2_rating = get_rating(
        diagnostic_report=diagnostic_report,
        tie_breaker='0',
        commonness_selector=min,
    )
    return oxygen_rating * co2_rating


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

    # Reformat in a more workable format
    diagnostic_report = pd.DataFrame([iter(l.rstrip()) for l in sys.stdin])
    print(fn(diagnostic_report))


if __name__ == '__main__':
    main()
