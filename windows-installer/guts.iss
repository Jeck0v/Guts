#include "config.iss"

[Setup]
AppName=Guts
AppVersion=0.0.6
AppPublisher=UNFAIR Team
AppPublisherURL=https://github.com/Jeck0v/Guts
AppSupportURL=https://github.com/Jeck0v/Guts/discussions
AppUpdatesURL=https://github.com/Jeck0v/Guts/releases
WizardSmallImageFile=guts.bmp
DefaultDirName={pf}\Guts
DefaultGroupName=Guts
OutputBaseFilename=Guts_Installer
OutputDir=.
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Files]
Source: "{#MyAppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "add-to-path.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MyAppIcon}"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcut :"

[Icons]
Name: "{group}\Guts Terminal"; Filename: "{app}\guts.exe"; IconFilename: "{app}\guts.ico"
Name: "{commondesktop}\Guts Terminal"; Filename: "{app}\guts.exe"; Tasks: desktopicon; IconFilename: "{app}\guts.ico"

[Run]
Filename: "powershell.exe"; \
  Parameters: "-ExecutionPolicy Bypass -NoProfile -File ""{app}\add-to-path.ps1"""; \
  Flags: runhidden

; ============================
; Section [Code] for custom UI
; ============================
[Code]

var
  AboutPage: TWizardPage;
  AboutLabel: TLabel;

procedure InitializeWizard;
begin
  AboutPage := CreateCustomPage(
    wpWelcome, // ou wpLicense, etc.
    'About Guts',
    'Information about the project'
  );

  AboutLabel := TLabel.Create(AboutPage);
  AboutLabel.Parent := AboutPage.Surface;
  AboutLabel.AutoSize := False;
  AboutLabel.WordWrap := True;
  AboutLabel.SetBounds(0, 0, AboutPage.SurfaceWidth, 100);
  AboutLabel.Caption :=
    'This TUI was developed by the UNFAIR team.' + #13#10 +
    'Feel free to add a star to the GitHub project:' + #13#10 +
    'https://github.com/Jeck0v/Guts' + #13#10 +
    'Thanks for using Guts!' + #13#10 +
    'Feel free to report issues or ideas in the GitHub discussions.';
end;
