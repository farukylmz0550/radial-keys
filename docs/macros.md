# Python macros

Each of the eight radial slots can have a label, icon, and Python script. Open the Radial Keys settings window, choose a `.py` file with **Browse…**, optionally choose a PNG, JPEG, or SVG icon, enter the label shown on the wheel, and press **Save**.

When a populated segment is clicked—or is highlighted and the activation shortcut is pressed again—the backend starts:

```text
python3 /absolute/path/to/your_macro.py
```

For example, a macro that opens Konsole can contain:

```python
import subprocess

subprocess.Popen(["konsole"])
```

The backend passes the selected script path as an argument and does not invoke a shell. A script runs as the logged-in user and has that user's permissions. Only assign scripts you trust. Python packages used by a macro must already be available to the `python3` interpreter.

An empty macro path leaves the slot as a no-op. Its label can still be changed. Slot labels are limited to 32 characters.
