Unicode true
RequestExecutionLevel user
ManifestDPIAware true

!include "MUI2.nsh"

!ifndef APP_VERSION
  !define APP_VERSION "0.1.32"
!endif
!define APP_NAME "M4 PC Monitor"
!define APP_EXE "M4PCMonitor.exe"
!define APP_STAGE "artifacts\M4PCMonitor-Portable-v${APP_VERSION}-x64"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\M4PCMonitor"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "artifacts\M4-PC-Monitor-Setup-${APP_VERSION}-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\M4 PC Monitor"
InstallDirRegKey HKCU "Software\M4PCMonitor" "InstallDir"
ShowInstDetails show
ShowUnInstDetails show

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Запустить M4 PC Monitor"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Russian"

Section "M4 PC Monitor" SEC_MAIN
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "${APP_STAGE}\M4PCMonitor.exe"
  File "${APP_STAGE}\M4PCMonitor.ico"
  File "${APP_STAGE}\hardware-sensors.ps1"
  File "${APP_STAGE}\LibreHardwareMonitorLib.dll"
  File "${APP_STAGE}\LICENSE-LibreHardwareMonitor.txt"
  File "${APP_STAGE}\README_Portable.md"
  CreateDirectory "$INSTDIR\fonts"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\M4PCMonitor" "InstallDir" "$INSTDIR"
  CreateDirectory "$SMPROGRAMS\M4 PC Monitor"
  CreateShortcut "$SMPROGRAMS\M4 PC Monitor\M4 PC Monitor.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\M4PCMonitor.ico"
  CreateShortcut "$DESKTOP\M4 PC Monitor.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\M4PCMonitor.ico"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "M4"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$SMSTARTUP\M4 PC Monitor.lnk"
  Delete "$DESKTOP\M4 PC Monitor.lnk"
  Delete "$SMPROGRAMS\M4 PC Monitor\M4 PC Monitor.lnk"
  RMDir "$SMPROGRAMS\M4 PC Monitor"
  Delete "$INSTDIR\${APP_EXE}"
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
SectionEnd
