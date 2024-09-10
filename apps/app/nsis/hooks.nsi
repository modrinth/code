!macro NSIS_HOOK_POSTINSTALL
    ; Check if theseus_gui.exe exists
    ${IfFileExists} "$LOCALAPPDATA\Modrinth App\theseus_gui.exe"

        Delete "$LOCALAPPDATA\Modrinth App\theseus_gui.exe"

        {{#each file_associations as |association| ~}}
          {{#each association.ext as |ext| ~}}
            !insertmacro APP_UNASSOCIATE "{{ext}}" "{{or association.name ext}}"
          {{/each}}
        {{/each}}

        {{#each deep_link_protocols as |protocol| ~}}
          ReadRegStr $R7 SHCTX "Software\Classes\\{{protocol}}\shell\open\command" ""
          ${If} $R7 == "$\"$LOCALAPPDATA\Modrinth App\theseus_gui.exe$\" $\"%1$\""
            DeleteRegKey SHCTX "Software\Classes\\{{protocol}}"
          ${EndIf}
        {{/each}}

        Delete "$LOCALAPPDATA\Modrinth App\uninstall.exe"
        RMDir "$LOCALAPPDATA\Modrinth App"

        !insertmacro DeleteAppUserModelId

        ; Remove start menu shortcut
        !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
        !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$LOCALAPPDATA\Modrinth App\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
          Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
          RMDir "$SMPROGRAMS\$AppStartMenuFolder"
        ${EndIf}
        !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$LOCALAPPDATA\Modrinth App\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
          Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
        ${EndIf}

        ; Remove desktop shortcuts
        !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$LOCALAPPDATA\Modrinth App\theseus_gui.exe"
        Pop $0
        ${If} $0 = 1
          !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
          Delete "$DESKTOP\${PRODUCTNAME}.lnk"
        ${EndIf}

        DeleteRegKey HKCU "${UNINSTKEY}"

    ${EndIfFileExists} ; End of file existence check

!macroend
