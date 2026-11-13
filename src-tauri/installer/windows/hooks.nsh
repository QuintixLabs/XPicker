; ============================================================
; hooks.nsh
; Extra steps for the Windows installer and uninstaller.
; ============================================================

; Up to 2.2.0 the program was xpicker.exe. Windows keeps a file's old name when
; it's replaced, so rename it first and the update lands as XPicker.exe. Start
; with computer and shortcuts still find it: Windows paths ignore case.
!macro NSIS_HOOK_PREINSTALL
  ${If} ${FileExists} "$INSTDIR\xpicker.exe"
    Rename "$INSTDIR\xpicker.exe" "$INSTDIR\XPicker.exe"
  ${EndIf}
!macroend

; Tauri's "delete data" checkbox only deletes folders named after the app's ID,
; but XPicker keeps its things in "XPicker" folders. So when the box is ticked,
; also delete those: the settings and recent colors (Roaming) and the browser
; data and color squares (Local). Never while updating.
;
; XPicker also introduces itself to Windows so its notifications show
; (tray/notifications.rs). Remove that on a real uninstall, ticked or not.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegKey HKCU "Software\Classes\AppUserModelId\${BUNDLEID}"
  ${EndIf}

  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    RmDir /r "$APPDATA\${PRODUCTNAME}"
    RmDir /r "$LOCALAPPDATA\${PRODUCTNAME}"
  ${EndIf}
!macroend
