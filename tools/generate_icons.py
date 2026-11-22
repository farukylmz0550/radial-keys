#!/usr/bin/env python3
"""Generate the original transparent white Radial Keys SVG icon set."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ICON_DIR = ROOT / "assets" / "icons"

ICONS = {
    "open-website.svg": '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a15 15 0 0 1 0 18M12 3a15 15 0 0 0 0 18"/>',
    "launch-application.svg": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18M7 6.5h.01M10 6.5h.01M14 14h4m-2-2 2 2-2 2"/>',
    "run-command.svg": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18m-13 4 2 2-2 2m5 0h4"/>',
    "open-folder.svg": '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M3 10h18"/>',
    "screenshot-full.svg": '<rect x="4" y="4" width="16" height="16" rx="2"/><path d="M8 2v4m8-4v4M8 18v4m8-4v4M2 8h4m12 0h4M2 16h4m12 0h4"/>',
    "screenshot-region.svg": '<path d="M4 9V5a1 1 0 0 1 1-1h4m6 0h4a1 1 0 0 1 1 1v4m0 6v4a1 1 0 0 1-1 1h-4m-6 0H5a1 1 0 0 1-1-1v-4"/><path d="M8 8h8v8H8z"/>',
    "media-play-pause.svg": '<path d="M5 4.5a1 1 0 0 1 1.5-.86l12 7a1.55 1.55 0 0 1 0 2.72l-12 7A1 1 0 0 1 5 19.5z"/><path d="M9 9v6m4-6v6"/>',
    "media-next.svg": '<path d="M4 5v14l10-7zM20 5v14"/>',
    "media-previous.svg": '<path d="M20 5v14L10 12zM4 5v14"/>',
    "media-stop.svg": '<rect x="5" y="5" width="14" height="14" rx="2"/>',
    "volume-toggle.svg": '<path d="M4 10v4h4l5 4V6l-5 4z"/><path d="m17 9 4 6m0-6-4 6"/>',
    "volume-up.svg": '<path d="M4 10v4h4l5 4V6l-5 4z"/><path d="M17 9a5 5 0 0 1 0 6m2-9a9 9 0 0 1 0 12"/>',
    "volume-down.svg": '<path d="M4 10v4h4l5 4V6l-5 4z"/><path d="M17 10a3 3 0 0 1 0 4"/>',
    "microphone-muted.svg": '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 12 5m2-5a7 7 0 0 1-.7 3M12 18v3m-4 0h8M3 3l18 18"/>',
    "microphone-unmuted.svg": '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3m-4 0h8"/>',
    "lock-screen.svg": '<rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3m-4 4v3"/>',
    "show-desktop.svg": '<rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8m-4-4v4m-5-9 3-3 2 2 3-4 2 2"/>',
}

def svg(body: str) -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" '
        'fill="none" stroke="#ffffff" stroke-width="1.8" '
        'stroke-linecap="round" stroke-linejoin="round">'
        f"{body}</svg>\n"
    )


def main() -> None:
    ICON_DIR.mkdir(parents=True, exist_ok=True)
    for name, body in ICONS.items():
        (ICON_DIR / name).write_text(svg(body), encoding="utf-8")


if __name__ == "__main__":
    main()
