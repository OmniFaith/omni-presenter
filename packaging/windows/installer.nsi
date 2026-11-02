Unicode true
!include "MUI2.nsh"

Name "Omni Presenter ${VERSION}"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\Omni Presenter"
RequestExecutionLevel user

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${PROJECT_ROOT}\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  File /oname=omni-presenter.exe "${BINARY}"
  File /oname=LICENSE "${PROJECT_ROOT}\LICENSE"
  File /oname=operations.md "${PROJECT_ROOT}\docs\operations.md"
  SetOutPath "$INSTDIR\licenses"
  File /oname=bible-source-LICENSE "${PROJECT_ROOT}\assets\bible-source\LICENSE"
  File /oname=NotoEmoji-OFL.txt "${PROJECT_ROOT}\assets\fonts\OFL.txt"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "DisplayName" "Omni Presenter"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "DisplayIcon" "$INSTDIR\omni-presenter.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter" "NoRepair" 1
  CreateDirectory "$LOCALAPPDATA\Omni Presenter\workspace"
  SetOutPath "$LOCALAPPDATA\Omni Presenter\workspace"
  CreateDirectory "$SMPROGRAMS\Omni Presenter"
  CreateShortCut "$SMPROGRAMS\Omni Presenter\Omni Presenter.lnk" "$INSTDIR\omni-presenter.exe"
  CreateShortCut "$SMPROGRAMS\Omni Presenter\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortCut "$DESKTOP\Omni Presenter.lnk" "$INSTDIR\omni-presenter.exe"
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  Delete "$SMPROGRAMS\Omni Presenter\Omni Presenter.lnk"
  Delete "$SMPROGRAMS\Omni Presenter\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Omni Presenter"
  Delete "$DESKTOP\Omni Presenter.lnk"
  Delete "$INSTDIR\omni-presenter.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\operations.md"
  Delete "$INSTDIR\licenses\bible-source-LICENSE"
  Delete "$INSTDIR\licenses\NotoEmoji-OFL.txt"
  RMDir "$INSTDIR\licenses"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OmniPresenter"
SectionEnd
