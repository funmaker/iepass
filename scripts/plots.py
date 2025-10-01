import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('../lua/numbers.csv', dtype=str)
fig, axes = plt.subplots(5, 2, figsize=(15, 12))
axes = axes.flatten()

def neg(val):
    if val >= 0x8000_0000:
        return val - 0xFFFF_FFFF
    else:
        return val

y_columns = [
    'Sin', 'Cos',
    'atan2-ne', 'atan2-nw',
    'atan2-sw', 'atan2-se',
    'x^2', '2^x',
    'Sqrt', 'Dec',
]

for y_col in y_columns + ['Hex']:
    if y_col == 'Dec':
        df[y_col] = df[y_col].apply(lambda x: int(x.replace(".", '').ljust(5, '0'), 10))
    else:
        df[y_col] = df[y_col].apply(lambda x: neg(int(x.replace(".", ''), 16)))

for i, y_col in enumerate(y_columns):
    axes[i].scatter(df['Hex'], df[y_col], marker='x', alpha=0.25)
    axes[i].set_title(y_col)
    axes[i].set_xticks(np.linspace(0, 2**16, 9), np.linspace(0, 1, 9))

plt.tight_layout()
plt.show()
