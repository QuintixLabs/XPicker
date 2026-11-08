; ============================================================
; hooks.nsh
; Extra steps for the Windows installer and uninstaller.
; ============================================================

; Tauri's "delete data" checkbox only deletes folders named after the app's ID,
; but XPicker keeps its things in "XPicker" folders. So when the box is ticked,
; also delete those: the settings and recent colors (Roaming) and the browser
; data and color squares (Local). Never while updating.
;
; XPicker also introduces itself to Windows so its notifications show
; (notifications.rs). Remove that on a real uninstall, ticked or not.
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
