!macro NSIS_HOOK_POSTINSTALL
    SetShellVarContext current

    IfFileExists "$LOCALAPPDATA${PRODUCTNAME}\theseus_gui.exe" file_found file_not_found
    file_found:
        Delete "$LOCALAPPDATA${PRODUCTNAME}\theseus_gui.exe"

        Delete "$LOCALAPPDATA${PRODUCTNAME}\uninstall.exe"
        RMDir "$LOCALAPPDATA${PRODUCTNAME}"

        !insertmacro DeleteAppUserModelId

        ; Remove start menu shortcut
        !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
        !insertmacro IsShortcutTarget "$SMPROGRAMS$AppStartMenuFolder${PRODUCTNAME}.lnk" "$LOCALAPPDATA${PRODUCTNAME}\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$SMPROGRAMS$AppStartMenuFolder${PRODUCTNAME}.lnk"
          Delete "$SMPROGRAMS$AppStartMenuFolder${PRODUCTNAME}.lnk"
          RMDir "$SMPROGRAMS$AppStartMenuFolder"
        ${EndIf}
        !insertmacro IsShortcutTarget "$SMPROGRAMS${PRODUCTNAME}.lnk" "$LOCALAPPDATA${PRODUCTNAME}\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$SMPROGRAMS${PRODUCTNAME}.lnk"
          Delete "$SMPROGRAMS${PRODUCTNAME}.lnk"
        ${EndIf}

        !insertmacro IsShortcutTarget "$DESKTOP${PRODUCTNAME}.lnk" "$LOCALAPPDATA${PRODUCTNAME}\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$DESKTOP${PRODUCTNAME}.lnk"
          Delete "$DESKTOP${PRODUCTNAME}.lnk"
        ${EndIf}

        DeleteRegKey HKCU "${UNINSTKEY}"

        goto end_of_test ;<== important for not continuing on the else branch
    file_not_found:
    end_of_test:
!macroend
