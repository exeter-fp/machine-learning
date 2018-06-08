# Setup Instructions

## Environment Creation and Dependencies

Run the following commands to set up the environment.

We need to use `venv` instead of `virtualenv` because Matplotlib doesn't work
with `virtualenv` as described [here](https://matplotlib.org/faq/osx_framework.html).

```bash
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
```


## Testing Matplotlib in IPython

To test Matplotlib in IPython, open a new IPython shell and run:

```
%matplotlib
```

If everything is working, you should get the following response:

```
Using matplotlib backend: MacOSX
```

Then run the following commands:

```python
import numpy as np
import matplotlib.pyplot as plt
plt.plot(np.random.randn(50))
```

A plot window should appear with a plot of random numbers.