#define MyAppName "SilliReminder"
#define MyAppVersion "1.0.0"
#define MyAppExeName "SilliReminder.exe"

; Fill this with the direct download URL to your GitHub Release asset (.exe)
#define MyAppUrl "https://github.com/Fanoks/SilliReminder/releases/download/v1.0.0/SilliReminder.exe"

; Optional: extra instructions file (downloaded to Desktop if the user checks the task)
; Fill this with the direct download URL to your GitHub Release asset (e.g. .pdf)
; If you have only one file for all languages, fill MyExtraInstructionsUrl and leave the language-specific URLs empty.
#define MyExtraInstructionsUrl "https://github.com/Fanoks/SilliReminder/releases/download/v1.0.0/instruction.pdf"
#define MyExtraInstructionsUrl_en "https://github.com/Fanoks/SilliReminder/releases/download/v1.0.0/instruction.pdf"
#define MyExtraInstructionsUrl_pl "https://github.com/Fanoks/SilliReminder/releases/download/v1.0.0/instrukcja.pdf"
; The file name that will be saved to Desktop
#define MyExtraInstructionsFileName "SilliReminder - Instructions.md"
;   (Get-FileHash .\SilliReminder.exe -Algorithm SHA256).Hash
#define MyAppSha256 "41d016cd199a849d82ecc79fee395d0af328b808a667549f04ffa121a04ce426"

; This installer uses the Inno Download Plugin (IDP) to download the EXE.
; Install IDP, then compile this script with Inno Setup.
#define _ScriptDir ExtractFilePath(__PATHFILENAME__)
#pragma include __INCLUDE__ + ";" + _ScriptDir
#include "idp\\idp.iss"

[Setup]
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputBaseFilename={#MyAppName}-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "pl"; MessagesFile: "compiler:Languages\\Polish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:TaskDesktopIcon}"; GroupDescription: "{cm:TaskIconsGroup}"; Flags: unchecked
Name: "extrainstructions"; Description: "{cm:TaskExtraInstructions}"; GroupDescription: "{cm:TaskDownloadsGroup}"; Flags: unchecked

[Icons]
Name: "{autoprograms}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"
Name: "{autodesktop}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"; Tasks: desktopicon

[Dirs]
Name: "{app}"; Flags: uninsalwaysuninstall

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
en.TaskExtraInstructions=Download extra instructions to Desktop
en.RunLaunchApp=Open {#MyAppName} after closing the wizard

pl.UninstallDataGroup=Opcjonalne czyszczenie:
pl.UninstallRemoveDb=Usuń bazę danych (przypomnienia)
pl.UninstallRemoveSettings=Usuń ustawienia (preferencje)

pl.TaskIconsGroup=Dodatkowe ikony:
pl.TaskDesktopIcon=Utwórz ikonę na &pulpicie
pl.TaskDownloadsGroup=Dodatkowe pobieranie:
pl.TaskExtraInstructions=Pobierz dodatkową instrukcję na Pulpit
pl.RunLaunchApp=Otwórz {#MyAppName} po zamknięciu kreatora

[UninstallDelete]
Type: files; Name: "{app}\\{#MyAppExeName}"
Type: dirifempty; Name: "{app}"

[Run]
Filename: "{app}\\{#MyAppExeName}"; Description: "{cm:RunLaunchApp}"; Flags: nowait postinstall skipifsilent

[Code]
var
  DownloadTarget: string;
  ExtraInstructionsTarget: string;
  UninstallRemoveDbSelected: Boolean;
  UninstallRemoveSettingsSelected: Boolean;

function GetLastError: Cardinal;
  external 'GetLastError@kernel32.dll stdcall';

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

function UninstallIsSilent: Boolean;
var
  I: Integer;
  P: string;
begin
  for I := 1 to ParamCount do
  begin
    P := UpperCase(ParamStr(I));
    if (P = '/SILENT') or (P = '/VERYSILENT') then
    begin
      Result := True;
      exit;
    end;
  end;

  Result := False;
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
  OutText: AnsiString;
  S: string;
  P: Integer;
  Token: string;
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

  S := string(OutText);
  StringChangeEx(S, #13, ' ', True);
  StringChangeEx(S, #10, ' ', True);
  StringChangeEx(S, #9, ' ', True);

  while Length(S) > 0 do
  begin
    while (Length(S) > 0) and (S[1] = ' ') do
      Delete(S, 1, 1);

    if Length(S) = 0 then
      break;

    P := Pos(' ', S);
    if P = 0 then
    begin
      Token := S;
      S := '';
    end
    else
    begin
      Token := Copy(S, 1, P - 1);
      Delete(S, 1, P);
    end;

    Hash := NormalizeHex(Token);
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

function SelectedExtraInstructionsUrl(): string;
var
  Lang: string;
begin
  Lang := Lowercase(Trim(ExpandConstant('{language}')));

  if (Lang = 'pl') and ('{#MyExtraInstructionsUrl_pl}' <> '') then
    Result := '{#MyExtraInstructionsUrl_pl}'
  else if (Lang = 'en') and ('{#MyExtraInstructionsUrl_en}' <> '') then
    Result := '{#MyExtraInstructionsUrl_en}'
  else
    Result := '{#MyExtraInstructionsUrl}';
end;

function ExtraInstructionsDesktopPath(): string;
begin
  Result := ExpandConstant('{userdesktop}\\{#MyExtraInstructionsFileName}');
end;

procedure InitializeWizard;
begin
  DownloadTarget := ExpandConstant('{tmp}\\{#MyAppExeName}');
  ExtraInstructionsTarget := ExpandConstant('{tmp}\\extra_instructions');

  idpDownloadAfter(wpReady);
end;

procedure InitializeUninstallProgressForm;
var
  Form: TSetupForm;
  Info: TNewStaticText;
  RemoveDb: TNewCheckBox;
  RemoveSettings: TNewCheckBox;
  OkBtn: TNewButton;
  CancelBtn: TNewButton;
  Res: Integer;
begin
  UninstallRemoveDbSelected := False;
  UninstallRemoveSettingsSelected := False;

  { Uninstall can finish very fast; collect choices before it starts }
  if UninstallIsSilent then
    exit;

  Form := CreateCustomForm(ScaleX(420), ScaleY(180), False, True);
  Form.Caption := ExpandConstant('{#MyAppName}');

  Info := TNewStaticText.Create(Form);
  Info.Parent := Form;
  Info.Left := ScaleX(12);
  Info.Top := ScaleY(12);
  Info.Width := Form.ClientWidth - ScaleX(24);
  Info.Height := ScaleY(32);
  Info.Caption := CustomMessage('UninstallDataGroup');
  Info.Font.Style := [fsBold];

  RemoveDb := TNewCheckBox.Create(Form);
  RemoveDb.Parent := Form;
  RemoveDb.Left := ScaleX(20);
  RemoveDb.Top := Info.Top + Info.Height + ScaleY(8);
  RemoveDb.Width := Form.ClientWidth - ScaleX(40);
  RemoveDb.Caption := CustomMessage('UninstallRemoveDb');
  RemoveDb.Checked := False;

  RemoveSettings := TNewCheckBox.Create(Form);
  RemoveSettings.Parent := Form;
  RemoveSettings.Left := ScaleX(20);
  RemoveSettings.Top := RemoveDb.Top + RemoveDb.Height + ScaleY(6);
  RemoveSettings.Width := Form.ClientWidth - ScaleX(40);
  RemoveSettings.Caption := CustomMessage('UninstallRemoveSettings');
  RemoveSettings.Checked := False;

  OkBtn := TNewButton.Create(Form);
  OkBtn.Parent := Form;
  OkBtn.Caption := SetupMessage(msgButtonOK);
  OkBtn.ModalResult := mrOk;
  OkBtn.Default := True;
  OkBtn.Left := Form.ClientWidth - ScaleX(180);
  OkBtn.Top := Form.ClientHeight - ScaleY(44);
  OkBtn.Width := ScaleX(80);

  CancelBtn := TNewButton.Create(Form);
  CancelBtn.Parent := Form;
  CancelBtn.Caption := SetupMessage(msgButtonCancel);
  CancelBtn.ModalResult := mrCancel;
  CancelBtn.Cancel := True;
  CancelBtn.Left := Form.ClientWidth - ScaleX(92);
  CancelBtn.Top := Form.ClientHeight - ScaleY(44);
  CancelBtn.Width := ScaleX(80);

  Res := Form.ShowModal;
  if Res <> mrOk then
    Abort;

  UninstallRemoveDbSelected := RemoveDb.Checked;
  UninstallRemoveSettingsSelected := RemoveSettings.Checked;
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

    { Build the download list right before the IDP download page }
    idpClearFiles;
    idpAddFile('{#MyAppUrl}', DownloadTarget);

    if WizardIsTaskSelected('extrainstructions') then
    begin
      if SelectedExtraInstructionsUrl() = '<FILL_ME_GITHUB_DOWNLOAD_URL_TO_INSTRUCTIONS>' then
      begin
        MsgBox('Installer is not configured. Set MyExtraInstructionsUrl (or MyExtraInstructionsUrl_en/pl) in the .iss file (or uncheck the extra instructions task).', mbError, MB_OK);
        Result := False;
        exit;
      end;

      if not UrlLooksSecure(SelectedExtraInstructionsUrl()) then
      begin
        MsgBox('Unsafe extra instructions URL. Only https:// URLs are allowed.', mbError, MB_OK);
        Result := False;
        exit;
      end;

      idpAddFile(SelectedExtraInstructionsUrl(), ExtraInstructionsTarget);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    if not FileExists(DownloadTarget) then
    begin
      MsgBox('The application file was not downloaded. Installation cannot continue.', mbError, MB_OK);
      Abort;
    end;

    if not VerifyDownloadedExe(DownloadTarget) then
      Abort;

    if not ForceDirectories(ExpandConstant('{app}')) then
    begin
      MsgBox('Failed to create the install directory:' + #13#10 + ExpandConstant('{app}'), mbError, MB_OK);
      Abort;
    end;

    if not CopyFile(DownloadTarget, ExpandConstant('{app}\\{#MyAppExeName}'), False) then
    begin
      MsgBox(
        'Failed to install the application file.' + #13#10 +
        'From: ' + DownloadTarget + #13#10 +
        'To:   ' + ExpandConstant('{app}\\{#MyAppExeName}') + #13#10 +
        'Error: ' + IntToStr(GetLastError) + ' - ' + SysErrorMessage(GetLastError),
        mbError,
        MB_OK
      );
      Abort;
    end;

    if WizardIsTaskSelected('extrainstructions') then
    begin
      if not FileExists(ExtraInstructionsTarget) then
      begin
        MsgBox('Extra instructions were not downloaded. Installation will continue without them.', mbInformation, MB_OK);
      end
      else if not CopyFile(ExtraInstructionsTarget, ExtraInstructionsDesktopPath(), False) then
      begin
        MsgBox(
          'Failed to save extra instructions to Desktop.' + #13#10 +
          'To:   ' + ExtraInstructionsDesktopPath() + #13#10 +
          'Error: ' + IntToStr(GetLastError) + ' - ' + SysErrorMessage(GetLastError),
          mbInformation,
          MB_OK
        );
      end;
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
  begin
    if UninstallRemoveDbSelected then
    begin
      BestEffortDeleteFile(DbPath());
      BestEffortRemoveDirIfEmpty(AppDataDir() + '\\data');
    end;

    if UninstallRemoveSettingsSelected then
    begin
      BestEffortDeleteFile(SettingsPath());
    end;

    { If we removed something, try to clean up empty app data folder }
    BestEffortRemoveDirIfEmpty(AppDataDir());
  end;
end;
