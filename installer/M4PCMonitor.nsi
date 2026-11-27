Unicode true
RequestExecutionLevel user
ManifestDPIAware true

!include "MUI2.nsh"
!include "LogicLib.nsh"

!ifndef APP_VERSION
  !define APP_VERSION "0.1.44"
!endif
!define APP_NAME "M4Dash"
!define APP_EXE "M4Dash.exe"
!define APP_STAGE "artifacts\M4Dash-Portable-v${APP_VERSION}-x64"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\M4PCMonitor"
!define MUI_ICON "assets\M4PCMonitor.ico"
!define MUI_UNICON "assets\M4PCMonitor.ico"
LangString DesktopShortcutSection 1033 "Create a desktop shortcut"
LangString DesktopShortcutSection 1049 "Создать ярлык на рабочем столе"
LangString DesktopShortcutDescription 1033 "Add or remove the M4Dash shortcut on the desktop."
LangString DesktopShortcutDescription 1049 "Добавить или убрать ярлык M4Dash на рабочем столе."

Name "${APP_NAME} ${APP_VERSION}"
OutFile "artifacts\M4Dash-Setup-${APP_VERSION}-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\M4Dash"
InstallDirRegKey HKCU "Software\M4PCMonitor" "InstallDir"
ShowInstDetails show
ShowUnInstDetails show

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Запустить M4Dash"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Russian"

Var PreviousInstallDir
Var LegacyInstallDir
Function .onInit
  StrCpy $PreviousInstallDir $INSTDIR
  StrCpy $LegacyInstallDir "$LOCALAPPDATA\Programs\M4 PC Monitor"
  StrCmp $INSTDIR "$LegacyInstallDir" 0 init_done
  StrCpy $INSTDIR "$LOCALAPPDATA\Programs\M4Dash"
init_done:
FunctionEnd
Function un.onInit
  StrCpy $LegacyInstallDir "$LOCALAPPDATA\Programs\M4 PC Monitor"
FunctionEnd

Section "M4Dash" SEC_MAIN
  SectionIn RO
  ${If} ${FileExists} "$LegacyInstallDir\settings.json"
    ${IfNot} ${FileExists} "$INSTDIR\settings.json"
      CopyFiles /SILENT "$LegacyInstallDir\settings.json" "$INSTDIR\settings.json"
    ${EndIf}
  ${EndIf}
  ${If} ${FileExists} "$PreviousInstallDir\settings.json"
    ${IfNot} ${FileExists} "$INSTDIR\settings.json"
      CopyFiles /SILENT "$PreviousInstallDir\settings.json" "$INSTDIR\settings.json"
    ${EndIf}
  ${EndIf}
  CreateDirectory "$INSTDIR\fonts"
  ${If} ${FileExists} "$LegacyInstallDir\fonts\*.*"
    CopyFiles /SILENT "$LegacyInstallDir\fonts\*.*" "$INSTDIR\fonts"
  ${EndIf}
  ${If} $PreviousInstallDir != $INSTDIR
    ${If} ${FileExists} "$PreviousInstallDir\fonts\*.*"
      CopyFiles /SILENT "$PreviousInstallDir\fonts\*.*" "$INSTDIR\fonts"
    ${EndIf}
  ${EndIf}
  SetOutPath "$INSTDIR"
  File "${APP_STAGE}\M4Dash.exe"
  File "${APP_STAGE}\M4PCMonitor.ico"
  File "${APP_STAGE}\hardware-sensors.ps1"
  File "${APP_STAGE}\LibreHardwareMonitorLib.dll"
  File "${APP_STAGE}\LICENSE-LibreHardwareMonitor.txt"
  File "${APP_STAGE}\README_Portable.md"
  ${If} ${FileExists} "$PreviousInstallDir\M4PCMonitor.exe"
    CopyFiles /SILENT "$INSTDIR\M4Dash.exe" "$PreviousInstallDir\M4PCMonitor.exe"
    SetFileAttributes "$PreviousInstallDir\M4PCMonitor.exe" HIDDEN
    ${If} $PreviousInstallDir != $INSTDIR
      SetFileAttributes "$PreviousInstallDir" HIDDEN
    ${EndIf}
  ${EndIf}
  ${If} $LegacyInstallDir != $PreviousInstallDir
    ${If} ${FileExists} "$LegacyInstallDir\M4PCMonitor.exe"
      CopyFiles /SILENT "$INSTDIR\M4Dash.exe" "$LegacyInstallDir\M4PCMonitor.exe"
      SetFileAttributes "$LegacyInstallDir\M4PCMonitor.exe" HIDDEN
      SetFileAttributes "$LegacyInstallDir" HIDDEN
    ${EndIf}
  ${EndIf}
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\M4PCMonitor" "InstallDir" "$INSTDIR"
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "M4PCMonitor"
  StrCmp $0 "" skip_autostart_migration
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "M4PCMonitor" "$\"$INSTDIR\${APP_EXE}$\""
skip_autostart_migration:
  Delete "$DESKTOP\M4 PC Monitor.lnk"
  Delete "$DESKTOP\M4Dash.lnk"
  Delete "$SMPROGRAMS\M4 PC Monitor\M4 PC Monitor.lnk"
  RMDir "$SMPROGRAMS\M4 PC Monitor"
  CreateDirectory "$SMPROGRAMS\M4Dash"
  CreateShortcut "$SMPROGRAMS\M4Dash\M4Dash.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\M4PCMonitor.ico"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\M4PCMonitor.ico"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "M4"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "$(DesktopShortcutSection)" SEC_DESKTOP
  CreateShortcut "$DESKTOP\M4Dash.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\M4PCMonitor.ico"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_DESKTOP} "$(DesktopShortcutDescription)"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  Delete "$SMSTARTUP\M4 PC Monitor.lnk"
  Delete "$SMSTARTUP\M4Dash.lnk"
  Delete "$DESKTOP\M4 PC Monitor.lnk"
  Delete "$DESKTOP\M4Dash.lnk"
  Delete "$SMPROGRAMS\M4 PC Monitor\M4 PC Monitor.lnk"
  Delete "$SMPROGRAMS\M4Dash\M4Dash.lnk"
  RMDir "$SMPROGRAMS\M4 PC Monitor"
  RMDir "$SMPROGRAMS\M4Dash"
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\M4PCMonitor.exe"
  Delete "$INSTDIR\M4PCMonitor.ico"
  Delete "$INSTDIR\hardware-sensors.ps1"
  Delete "$INSTDIR\LibreHardwareMonitorLib.dll"
  Delete "$INSTDIR\LICENSE-LibreHardwareMonitor.txt"
  Delete "$INSTDIR\README_Portable.md"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$INSTDIR\settings.json"
  RMDir /r "$INSTDIR\fonts"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "Software\M4PCMonitor"
  IfFileExists "$LegacyInstallDir\M4PCMonitor.exe" 0 +2
  RMDir /r "$LegacyInstallDir"
SectionEnd
