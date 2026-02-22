import os
import math
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.ticker import FormatStrFormatter, MaxNLocator

abspath = os.path.abspath(__file__)
dname = os.path.dirname(abspath)
os.chdir(dname)

y_columns = [
    'Sin', 'Cos',
    'atan2-ne', 'atan2-nw',
    'atan2-sw', 'atan2-se',
    'x^2', '2^x',
    'Sqrt', 'ToDecimal',
]

def rot_diff(val):
    if val > 0x8000:
        return val - 0x1_0000
    elif val < -0x8000:
        return val + 0x1_0000
    else:
        return val

def signed(val):
    if val > 0x8000_0000:
        return val - 0x1_0000_0000
    else:
        return val

def load_csv(path):
    df = pd.read_csv(path, dtype=str)
    
    for y_col in y_columns + ['Hex']:
        if y_col == 'Dec':
            df[y_col] = df[y_col].apply(lambda x: int(x.replace(".", '').ljust(5, '0'), 10))
        else:
            df[y_col] = df[y_col].apply(lambda x: signed(int(x.replace(".", ''), 16)))
    
    return df

pico8 = load_csv('./data/pico8.csv')
p8rs = load_csv('./data/p8rs.csv')

fig, axes = plt.subplots(math.ceil(len(y_columns) / 2), 2, figsize=(15, 12))
axes = axes.flatten()

for i, y_col in enumerate(y_columns):
    y = pico8[y_col] - p8rs[y_col]
    if y_col == "atan2-se":
        y = y.apply(rot_diff)
    axes[i].set_title(y_col)
    axes[i].set_xticks(np.linspace(0, 2**16, 9), np.linspace(0, 1, 9))
    axes[i].yaxis.set_major_formatter(FormatStrFormatter('%dε'))
    axes[i].yaxis.set_major_locator(MaxNLocator(integer=True))
    axes[i].scatter(pico8['Hex'], y, marker='x', alpha=0.25)
    y_np = y.to_numpy()
    if (y_np[0] == y_np).all():
        axes[i].set_yticks([y_np[0]])


plt.tight_layout()
plt.show()
