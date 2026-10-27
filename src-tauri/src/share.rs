//! Sends the phone link straight to the phone, through the system's own sharing.

/// Opens the `AirDrop` window with the link, then waits for it to close: the window belongs
/// to the script's process. A standalone window, because the panel hides on blur and a share
/// sheet anchored to it (`navigator.share`) vanished with it, leaving the panel stuck.
#[cfg(target_os = "macos")]
const AIRDROP_SCRIPT: &str = r"
ObjC.import('AppKit');
function run(argv) {
  const app = $.NSApplication.sharedApplication;
  app.setActivationPolicy($.NSApplicationActivationPolicyAccessory);
  const service = $.NSSharingService.sharingServiceNamed('com.apple.share.AirDrop.send');
  service.performWithItems($([$.NSURL.URLWithString(argv[0])]));
  app.activateIgnoringOtherApps(true);
  const visible = () => {
    const windows = app.windows;
    for (let i = 0; i < windows.count; i++) if (windows.objectAtIndex(i).isVisible) return true;
    return false;
  };
  const wait = () => $.NSRunLoop.currentRunLoop.runUntilDate($.NSDate.dateWithTimeIntervalSinceNow(0.2));
  for (let tries = 0; tries < 25 && !visible(); tries++) wait();
  while (visible()) wait();
}
";

#[cfg(target_os = "macos")]
pub fn send(url: &str) -> Result<(), String> {
    let mut child = std::process::Command::new("osascript")
        .args(["-l", "JavaScript", "-e", AIRDROP_SCRIPT, url])
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|error| format!("osascript: {error}"))?;
    // The script lives as long as the AirDrop window: reap it off the UI thread.
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn send(_url: &str) -> Result<(), String> {
    Err("sharing is only available on macOS".into())
}
