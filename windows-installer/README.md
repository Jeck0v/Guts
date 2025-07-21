# Guts Installer Overview

## add-to-path.ps1

This PowerShell script is executed during installation to add the `Guts installation folder` to the system-wide PATH environment variable.

- It reads the current machine PATH.
- Checks if the install path (`C:\Program Files\Guts`) is already included.
- If not, it appends the install path to the machine PATH.
- Logs the actions and results to a temporary log file (`%TEMP%\guts_install_log.txt`).

This allows you to run `guts.exe` from any command prompt without specifying the full path.

---

## guts.iss (Inno Setup Script)

This script builds the Windows installer for Guts using Inno Setup:

- Defines the application details (name, version, publisher, URLs).
- Specifies which files to include (the executable, icon, and the PowerShell script).
- Creates desktop and start menu shortcuts.
- Runs the `add-to-path.ps1` script silently after installation to update the PATH.
- Implements a custom installer page with information about the project and team.
- Requires administrator privileges for installation.

---

For more details, visit the project GitHub:
https://github.com/Jeck0v/Guts
