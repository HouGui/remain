; Compile from the repository root using scripts/package-windows.ps1.
#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef RepoRoot
  #error RepoRoot is required
#endif

[Setup]
AppId={{E727156F-AC11-49A4-9F8C-61A00AF7E3B7}
AppName=REMAIN
AppVersion={#AppVersion}
AppPublisher=REMAIN developers
DefaultDirName={localappdata}\Programs\REMAIN
DefaultGroupName=REMAIN
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
DisableProgramGroupPage=yes
LicenseFile={#RepoRoot}\LICENSE
OutputDir={#RepoRoot}\dist
OutputBaseFilename=remain-{#AppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\remain.exe
SetupIconFile={#RepoRoot}\packaging\remain.ico
CloseApplications=yes
SetupLogging=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#RepoRoot}\target\release\remain.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\site\*"; DestDir: "{app}\guide"; Flags: ignoreversion recursesubdirs createallsubdirs

[InstallDelete]
Type: filesandordirs; Name: "{app}\guide"

[Icons]
Name: "{autoprograms}\REMAIN"; Filename: "{app}\remain.exe"
Name: "{autodesktop}\REMAIN"; Filename: "{app}\remain.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\remain.exe"; Description: "{cm:LaunchProgram,REMAIN}"; Flags: nowait postinstall skipifsilent
