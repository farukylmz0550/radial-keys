use anyhow::{Context, anyhow};
use std::{env, path::PathBuf};
use tokio::process::Command;
use zbus::{Connection, Proxy};

const SCRIPT_NAME: &str = "radialkeysFullscreen";
const SCRIPT: &str = include_str!("kwin_fullscreen.js");
const WRITE_SCRIPT: &str = "import pathlib,sys; p=pathlib.Path(sys.argv[1]); p.parent.mkdir(parents=True,exist_ok=True); p.write_text(sys.argv[2])";

fn script_path() -> anyhow::Result<PathBuf> {
    let config_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| anyhow!("could not locate the user configuration directory"))?;
    Ok(config_dir.join("radial-keys/kwin/fullscreen.js"))
}

pub async fn install_fullscreen_monitor(connection: &Connection) -> anyhow::Result<()> {
    let path = script_path()?;
    let path_string = path.to_string_lossy().into_owned();
    let output = Command::new("flatpak-spawn")
        .args(["--host", "python3", "-c", WRITE_SCRIPT])
        .arg(&path_string)
        .arg(SCRIPT)
        .output()
        .await
        .context("could not start the host script installer")?;
    if !output.status.success() {
        return Err(anyhow!(
            "could not install the KWin fullscreen monitor: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let scripting = Proxy::new(
        connection,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )
    .await
    .context("could not connect to KWin scripting")?;
    let _ = scripting
        .call::<_, _, bool>("unloadScript", &(SCRIPT_NAME,))
        .await;
    let script_id: i32 = scripting
        .call("loadScript", &(path_string, SCRIPT_NAME))
        .await
        .context("KWin could not load the fullscreen monitor")?;
    let script = Proxy::new(
        connection,
        "org.kde.KWin",
        format!("/Scripting/Script{script_id}"),
        "org.kde.kwin.Script",
    )
    .await
    .context("could not connect to the loaded KWin script")?;
    script
        .call::<_, _, ()>("run", &())
        .await
        .context("KWin could not start the fullscreen monitor")?;
    Ok(())
}

pub async fn unload_fullscreen_monitor(connection: &Connection) {
    if let Ok(scripting) = Proxy::new(
        connection,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )
    .await
    {
        let _ = scripting
            .call::<_, _, bool>("unloadScript", &(SCRIPT_NAME,))
            .await;
    }
}
