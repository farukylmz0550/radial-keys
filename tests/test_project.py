"""Python unit, smoke, and end-to-end checks for the Rust/Qt project.

Run from the repository root with:
    dbus-run-session -- python3 -m unittest discover -s tests -v
"""

from __future__ import annotations

import ast
import json
import os
import signal
import subprocess
import tempfile
import time
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SERVICE = "io.github.radialkeys.Service"
OBJECT = "/io/github/radialkeys/Service"
INTERFACE = "io.github.radialkeys.Service1"


class IconUnitTests(unittest.TestCase):
    def test_action_icons_are_transparent_white_svg(self) -> None:
        icons = sorted((ROOT / "assets/icons").glob("*.svg"))
        self.assertEqual(len(icons), 17, "16 macros need 17 SVGs due to mic states")
        for icon in icons:
            with self.subTest(icon=icon.name):
                root = ET.parse(icon).getroot()
                self.assertEqual(root.tag.rsplit("}", 1)[-1], "svg")
                self.assertIsNone(root.get("style"), "SVG must not paint a background")
                self.assertEqual(root.get("fill"), "none")
                self.assertIn(root.get("stroke", "").lower(), ("#fff", "#ffffff", "white"))
                self.assertTrue(list(root), "icon must contain visible shapes")

    def test_microphone_icons_are_distinct_and_have_no_background(self) -> None:
        muted = (ROOT / "assets/icons/microphone-muted.svg").read_bytes()
        unmuted = (ROOT / "assets/icons/microphone-unmuted.svg").read_bytes()
        self.assertNotEqual(muted, unmuted)
        for source in (muted, unmuted):
            root = ET.fromstring(source)
            self.assertEqual(root.get("viewBox"), "0 0 24 24")
            self.assertTrue(all(child.tag.endswith(("path", "rect", "circle")) for child in root))


class BackendSmokeAndE2ETests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if not os.environ.get("DBUS_SESSION_BUS_ADDRESS"):
            raise RuntimeError("Run this suite inside dbus-run-session")
        cls.backend_binary = ROOT / "target/debug/radial-keys"
        cls.frontend_binary = ROOT / "build/frontend/radial-keys-frontend"
        if not cls.backend_binary.exists() or not cls.frontend_binary.exists():
            raise RuntimeError("Build backend and frontend before running the Python E2E suite")
        cls.temp = tempfile.TemporaryDirectory(prefix="radial-keys-e2e-")
        cls.temp_path = Path(cls.temp.name)
        cls.env = os.environ.copy()
        cls.env.update(
            XDG_CONFIG_HOME=str(cls.temp_path / "config"),
            XDG_DATA_HOME=str(cls.temp_path / "data"),
            QT_QPA_PLATFORM="offscreen",
            QT_QUICK_CONTROLS_STYLE="Basic",
        )
        (cls.temp_path / "config").mkdir()
        (cls.temp_path / "data").mkdir()
        cls.backend = subprocess.Popen(
            [str(cls.backend_binary)], env=cls.env, stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE, text=True
        )
        cls.frontend: subprocess.Popen[str] | None = None
        cls.wait_for_bus()

    @classmethod
    def tearDownClass(cls) -> None:
        if getattr(cls, "frontend", None) and cls.frontend.poll() is None:
            cls.frontend.terminate()
            try:
                cls.frontend.wait(timeout=3)
            except subprocess.TimeoutExpired:
                cls.frontend.kill()
                cls.frontend.wait(timeout=3)
        if getattr(cls, "backend", None) and cls.backend.poll() is None:
            cls.backend.send_signal(signal.SIGINT)
            try:
                cls.backend.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.backend.kill()
                cls.backend.wait(timeout=3)
        if getattr(cls, "temp", None):
            cls.temp.cleanup()

    @classmethod
    def wait_for_bus(cls) -> None:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if cls.backend.poll() is not None:
                raise AssertionError("backend exited before owning its D-Bus service")
            try:
                cls.call("Ping")
                return
            except (AssertionError, subprocess.CalledProcessError, FileNotFoundError):
                time.sleep(0.1)
        raise AssertionError("backend D-Bus service did not become ready")

    @classmethod
    def call(cls, method: str, *args: str) -> str:
        command = [
            "gdbus", "call", "--session", "--dest", SERVICE, "--object-path", OBJECT,
            "--method", f"{INTERFACE}.{method}", *args,
        ]
        result = subprocess.run(command, text=True, capture_output=True, timeout=5)
        if result.returncode:
            raise AssertionError(f"D-Bus {method} failed: {result.stderr.strip()}")
        return result.stdout.strip()

    @staticmethod
    def variant_string(value: str) -> str:
        return "'" + value.replace("\\", "\\\\").replace("'", "\\'").replace("\n", "\\n") + "'"

    @classmethod
    def returned_string(cls, output: str) -> str:
        # gdbus formats a single string as a one-element tuple.
        result = ast.literal_eval(output)
        if isinstance(result, tuple):
            return result[0]
        return result

    def test_backend_smoke_and_gui_macro_round_trip(self) -> None:
        self.assertEqual(self.returned_string(self.call("Ping")), "pong")
        version = self.returned_string(self.call("GetVersion"))
        self.assertTrue(version)
        catalog = json.loads(self.returned_string(self.call("GetBuiltinMacros")))
        self.assertEqual(len(catalog), 16)
        with self.assertRaisesRegex(AssertionError, "live renderer"):
            self.call("ActivateSlot", "0")
        with self.assertRaisesRegex(AssertionError, "must be absolute"):
            self.call("SetSlot", "0", self.variant_string("Invalid"), self.variant_string("relative.py"))
        self.assertEqual(self.call("SetMacroOptions", "true", "int64 -1"), "()")

        self.__class__.frontend = subprocess.Popen(
            [str(self.frontend_binary)], env=self.env, stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE, text=True
        )
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if self.frontend.poll() is not None:
                self.fail("Qt renderer exited during startup")
            status = json.loads(self.returned_string(self.call("GetStatus")))
            if status.get("renderer_available"):
                break
            time.sleep(0.1)
        else:
            self.fail("Qt renderer did not report its D-Bus heartbeat")

        sentinel = self.temp_path / "macro-ran.txt"
        source = f"from pathlib import Path\nPath({str(sentinel)!r}).write_text('ran')\n"
        self.call("SetSlotPython", "0", self.variant_string("Python test"),
                  self.variant_string(source), self.variant_string(""))
        slots = json.loads(self.returned_string(self.call("GetSlots")))
        self.assertEqual(slots[0]["script_integrity"], "verified")
        stored_script = Path(slots[0]["action"])
        self.assertTrue(stored_script.is_file())
        if os.name == "posix":
            self.assertEqual(stored_script.stat().st_mode & 0o777, 0o600)

        self.call("ShowMenu")
        self.call("GetStatus")  # Backend confirms the renderer observed the open menu.
        self.call("ActivateSlot", "0")
        self.wait_for_file(sentinel)
        self.assertEqual(sentinel.read_text(), "ran")

        stored_script.write_text(f"from pathlib import Path\nPath({str(sentinel)!r}).write_text('accepted')\n")
        self.assertEqual(json.loads(self.returned_string(self.call("GetSlots")))[0]["script_integrity"], "changed")
        self.call("ShowMenu")
        self.call("GetStatus")
        with self.assertRaisesRegex(AssertionError, "changed since it was accepted"):
            self.call("ActivateSlot", "0")
        self.call("AcceptScriptUpdate", "0")
        self.assertEqual(json.loads(self.returned_string(self.call("GetSlots")))[0]["script_integrity"], "verified")
        self.call("ShowMenu")
        self.call("GetStatus")
        self.call("ActivateSlot", "0")
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline and sentinel.read_text() != "accepted":
            time.sleep(0.05)
        self.assertEqual(sentinel.read_text(), "accepted")

    def wait_for_file(self, path: Path) -> None:
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if path.exists():
                return
            time.sleep(0.05)
        self.fail(f"macro did not produce expected output: {path}")


if __name__ == "__main__":
    unittest.main()
