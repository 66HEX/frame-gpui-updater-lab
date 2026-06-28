#define AppId "{{CCFCC551-7612-4D99-8F81-8F02DF450FDA}"
#ifndef AppName
#define AppName "Frame GPUI Lab"
#endif
#ifndef AppSetupName
#define AppSetupName "FrameGpuiLab-x86_64"
#endif
#ifndef AppVersion
#define AppVersion "0.1.0"
#endif
#ifndef OutputDir
#define OutputDir "..\target"
#endif
#ifndef ResourcesDir
#define ResourcesDir "..\target\inno\x86_64"
#endif

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=66HEX
AppPublisherURL=https://github.com/66HEX/frame-gpui-updater-lab
AppSupportURL=https://github.com/66HEX/frame-gpui-updater-lab/issues
AppUpdatesURL=https://github.com/66HEX/frame-gpui-updater-lab/releases
DefaultDirName={localappdata}\Programs\Frame GPUI Lab
DefaultGroupName=Frame GPUI Lab
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename={#AppSetupName}
SetupIconFile={#ResourcesDir}\app-icon.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\Frame.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#ResourcesDir}\Frame.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#ResourcesDir}\frame-update-helper.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#ResourcesDir}\*.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#ResourcesDir}\binaries\*"; DestDir: "{app}\binaries"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#ResourcesDir}\gstreamer\*"; DestDir: "{app}\gstreamer"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Frame GPUI Lab"; Filename: "{app}\Frame.exe"
Name: "{autodesktop}\Frame GPUI Lab"; Filename: "{app}\Frame.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Frame.exe"; Description: "{cm:LaunchProgram,Frame GPUI Lab}"; Flags: nowait postinstall skipifsilent
