#define MyAppName "SilliReminder"
#define MyAppVersion "1.0.0"
#define MyAppExeName "SilliReminder.exe"

; Fill this with the direct download URL to your GitHub Release asset (.exe)
#define MyAppUrl "<FILL_ME_GITHUB_DOWNLOAD_URL_TO_EXE>"

; Fill this with the expected SHA-256 of the downloaded EXE (64 hex chars)
; Example: "0123AB..." (no spaces). You can compute it with PowerShell:
;   (Get-FileHash .\SilliReminder.exe -Algorithm SHA256).Hash
#define MyAppSha256 "<FILL_ME_SHA256_OF_EXE>"

; This installer uses the Inno Download Plugin (IDP) to download the EXE.
; Install IDP, then compile this script with Inno Setup.
#include <idp.iss>

[Setup]
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputBaseFilename={#MyAppName}-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "pl"; MessagesFile: "compiler:Languages\\Polish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:TaskDesktopIcon}"; GroupDescription: "{cm:TaskIconsGroup}"; Flags: unchecked
Name: "extrainstructions"; Description: "{cm:TaskExtraInstructions}"; GroupDescription: "{cm:TaskDownloadsGroup}"; Flags: unchecked

[Icons]
Name: "{autoprograms}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"
Name: "{autodesktop}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; The app can enable autostart by writing this value itself.
; Ensure uninstall removes it even if it was created outside the installer.
Root: HKCU; Subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Run"; ValueType: string; ValueName: "SilliReminder"; ValueData: ""; Flags: uninsdeletevalue noerror

[CustomMessages]
en.UninstallDataGroup=Optional cleanup:
en.UninstallRemoveDb=Remove database (reminders)
en.UninstallRemoveSettings=Remove settings (preferences)

en.TaskIconsGroup=Additional icons:
en.TaskDesktopIcon=Create a &desktop icon
en.TaskDownloadsGroup=Additional downloads:
en.TaskExtraInstructions=Download extra instructions
en.RunLaunchApp=Open {#MyAppName} after closing the wizard

pl.UninstallDataGroup=Opcjonalne czyszczenie:
pl.UninstallRemoveDb=Usuń bazę danych (przypomnienia)
pl.UninstallRemoveSettings=Usuń ustawienia (preferencje)

pl.TaskIconsGroup=Dodatkowe ikony:
pl.TaskDesktopIcon=Utwórz ikonę na &pulpicie
pl.TaskDownloadsGroup=Dodatkowe pobieranie:
pl.TaskExtraInstructions=Pobierz dodatkową instrukcję
pl.RunLaunchApp=Otwórz {#MyAppName} po zamknięciu kreatora

[UninstallDelete]
Type: files; Name: "{app}\\{#MyAppExeName}"
Type: dirifempty; Name: "{app}"

[Run]
Filename: "{app}\\{#MyAppExeName}"; Description: "{cm:RunLaunchApp}"; Flags: nowait postinstall skipifsilent

[Code]
var
  DownloadPage: TDownloadWizardPage;
  DownloadTarget: string;

  UninstallCleanupGroup: TNewGroupBox;
  UninstallRemoveDbCheck: TNewCheckBox;
  UninstallRemoveSettingsCheck: TNewCheckBox;

function AppDataDir(): string;
begin
  Result := ExpandConstant('{localappdata}\\SilliReminder');
end;

function DbPath(): string;
begin
  Result := AppDataDir() + '\\data\\silli_reminder.db';
end;

function SettingsPath(): string;
begin
  Result := AppDataDir() + '\\settings.sillisettings';
end;

procedure BestEffortDeleteFile(const Path: string);
begin
  if FileExists(Path) then
  begin
    if not DeleteFile(Path) then
    begin
      MsgBox('Failed to remove file:' + #13#10 + Path + #13#10 +
        'It may be in use. Close the app and try uninstall again.', mbInformation, MB_OK);
    end;
  end;
end;

procedure BestEffortRemoveDirIfEmpty(const DirPath: string);
begin
  if DirExists(DirPath) then
  begin
    { Only removes if empty }
    RemoveDir(DirPath);
  end;
end;

function NormalizeHex(const S: string): string;
var
  I: Integer;
  C: Char;
begin
  Result := '';
  for I := 1 to Length(S) do
  begin
    C := S[I];
    if (C >= '0') and (C <= '9') then Result := Result + C
    else if (C >= 'a') and (C <= 'f') then Result := Result + Chr(Ord(C) - 32)
    else if (C >= 'A') and (C <= 'F') then Result := Result + C;
  end;
end;

function IsHexLen(const S: string; const N: Integer): Boolean;
var
  I: Integer;
  C: Char;
begin
  Result := False;
  if Length(S) <> N then exit;
  for I := 1 to Length(S) do
  begin
    C := S[I];
    if not (((C >= '0') and (C <= '9')) or ((C >= 'A') and (C <= 'F'))) then exit;
  end;
  Result := True;
end;

function GetFileSha256(const FileName: string): string;
var
  ResultCode: Integer;
  OutFile: string;
  OutText: string;
  Lines: TArrayOfString;
  I: Integer;
  L: string;
  Hash: string;
begin
  Result := '';

  OutFile := ExpandConstant('{tmp}\\sha256.txt');
  if FileExists(OutFile) then DeleteFile(OutFile);

  if not Exec(
    ExpandConstant('{cmd}'),
    '/C certutil -hashfile "' + FileName + '" SHA256 > "' + OutFile + '"',
    '',
    SW_HIDE,
    ewWaitUntilTerminated,
    ResultCode
  ) then
    exit;

  if ResultCode <> 0 then
    exit;

  if not LoadStringFromFile(OutFile, OutText) then
    exit;

  Lines := SplitString(OutText, #13#10);
  for I := 0 to GetArrayLength(Lines) - 1 do
  begin
    L := Trim(Lines[I]);
    Hash := NormalizeHex(L);
    if IsHexLen(Hash, 64) then
    begin
      Result := Hash;
      exit;
    end;
  end;
end;

function VerifyDownloadedExe(const FileName: string): Boolean;
var
  Expected: string;
  Actual: string;
begin
  Result := False;

  Expected := NormalizeHex('{#MyAppSha256}');
  if (Expected = '') or (Expected = 'FILLMESHA256OFEXE') or (Expected = '<FILLME_SHA256_OF_EXE>') then
  begin
    MsgBox('Installer is not configured. Set MyAppSha256 in the .iss file.', mbError, MB_OK);
    exit;
  end;

  if not IsHexLen(Expected, 64) then
  begin
    MsgBox('Installer SHA-256 value is invalid. It must be exactly 64 hex characters.', mbError, MB_OK);
    exit;
  end;

  Actual := GetFileSha256(FileName);
  if Actual = '' then
  begin
    MsgBox('Failed to compute SHA-256. (certutil not available?)', mbError, MB_OK);
    exit;
  end;

  if Actual <> Expected then
  begin
    DeleteFile(FileName);
    MsgBox(
      'Downloaded file hash mismatch!' + #13#10 +
      'Expected: ' + Expected + #13#10 +
      'Actual:   ' + Actual,
      mbError,
      MB_OK
    );
    exit;
  end;

  Result := True;
end;

function UrlLooksSecure(const Url: string): Boolean;
var
  U: string;
begin
  U := Lowercase(Trim(Url));
  Result := (Copy(U, 1, 8) = 'https://');
end;

procedure InitializeWizard;
begin
  DownloadTarget := ExpandConstant('{tmp}\\{#MyAppExeName}');

  DownloadPage := CreateDownloadPage(
    SetupMessage(msgWizardPreparing),
    'Downloading {#MyAppName}...',
    nil
  );

  DownloadPage.Add('{#MyAppUrl}', DownloadTarget, '');
end;

procedure InitializeUninstallProgressForm;
var
  LeftMargin: Integer;
  TopPos: Integer;
  WidthAvail: Integer;
begin
  { Add optional cleanup checkboxes into the uninstall wizard UI }
  LeftMargin := UninstallProgressForm.StatusLabel.Left;
  WidthAvail := UninstallProgressForm.ProgressBar.Width;
  TopPos := UninstallProgressForm.StatusLabel.Top + UninstallProgressForm.StatusLabel.Height + ScaleY(12);

  UninstallCleanupGroup := TNewGroupBox.Create(UninstallProgressForm);
  UninstallCleanupGroup.Parent := UninstallProgressForm.InnerNotebook;
  UninstallCleanupGroup.Left := LeftMargin;
  UninstallCleanupGroup.Top := TopPos;
  UninstallCleanupGroup.Width := WidthAvail;
  UninstallCleanupGroup.Height := ScaleY(72);
  UninstallCleanupGroup.Caption := CustomMessage('UninstallDataGroup');

  UninstallRemoveDbCheck := TNewCheckBox.Create(UninstallProgressForm);
  UninstallRemoveDbCheck.Parent := UninstallCleanupGroup;
  UninstallRemoveDbCheck.Left := ScaleX(10);
  UninstallRemoveDbCheck.Top := ScaleY(18);
  UninstallRemoveDbCheck.Width := UninstallCleanupGroup.Width - ScaleX(16);
  UninstallRemoveDbCheck.Caption := CustomMessage('UninstallRemoveDb');
  UninstallRemoveDbCheck.Checked := False;

  UninstallRemoveSettingsCheck := TNewCheckBox.Create(UninstallProgressForm);
  UninstallRemoveSettingsCheck.Parent := UninstallCleanupGroup;
  UninstallRemoveSettingsCheck.Left := ScaleX(10);
  UninstallRemoveSettingsCheck.Top := UninstallRemoveDbCheck.Top + UninstallRemoveDbCheck.Height + ScaleY(6);
  UninstallRemoveSettingsCheck.Width := UninstallCleanupGroup.Width - ScaleX(16);
  UninstallRemoveSettingsCheck.Caption := CustomMessage('UninstallRemoveSettings');
  UninstallRemoveSettingsCheck.Checked := False;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;

  if CurPageID = wpReady then
  begin
    if '{#MyAppUrl}' = '<FILL_ME_GITHUB_DOWNLOAD_URL_TO_EXE>' then
    begin
      MsgBox('Installer is not configured. Set MyAppUrl in the .iss file.', mbError, MB_OK);
      Result := False;
      exit;
    end;

    if not UrlLooksSecure('{#MyAppUrl}') then
    begin
      MsgBox('Unsafe download URL. Only https:// URLs are allowed.', mbError, MB_OK);
      Result := False;
      exit;
    end;

    if '{#MyAppSha256}' = '<FILL_ME_SHA256_OF_EXE>' then
    begin
      MsgBox('Installer is not configured. Set MyAppSha256 in the .iss file.', mbError, MB_OK);
      Result := False;
      exit;
    end;

    DownloadPage.Show;
    try
      DownloadPage.Download;
      if not VerifyDownloadedExe(DownloadTarget) then
      begin
        Result := False;
        exit;
      end;
      Result := True;
    except
      MsgBox('Download failed. Check your internet connection and URL.', mbError, MB_OK);
      Result := False;
    end;
    DownloadPage.Hide;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    if not FileCopy(DownloadTarget, ExpandConstant('{app}\\{#MyAppExeName}'), False) then
    begin
      MsgBox('Failed to install the application file.', mbError, MB_OK);
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
  begin
    if Assigned(UninstallRemoveDbCheck) and UninstallRemoveDbCheck.Checked then
    begin
      BestEffortDeleteFile(DbPath());
      BestEffortRemoveDirIfEmpty(AppDataDir() + '\\data');
    end;

    if Assigned(UninstallRemoveSettingsCheck) and UninstallRemoveSettingsCheck.Checked then
    begin
      BestEffortDeleteFile(SettingsPath());
    end;

    { If we removed something, try to clean up empty app data folder }
    BestEffortRemoveDirIfEmpty(AppDataDir());
  end;
end;
