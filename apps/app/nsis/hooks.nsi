; https://nsis.sourceforge.io/ShellExecWait
!macro ShellExecWait verb app param workdir show exitoutvar ;only app and show must be != "", every thing else is optional
    #define SEE_MASK_NOCLOSEPROCESS 0x40
    System::Store S
    !if "${NSIS_PTR_SIZE}" > 4
    !define /ReDef /math SYSSIZEOF_SHELLEXECUTEINFO 14 * ${NSIS_PTR_SIZE}
    !else ifndef SYSSIZEOF_SHELLEXECUTEINFO
    !define SYSSIZEOF_SHELLEXECUTEINFO 60
    !endif
    System::Call '*(&i${SYSSIZEOF_SHELLEXECUTEINFO})i.r0'
    System::Call '*$0(i ${SYSSIZEOF_SHELLEXECUTEINFO},i 0x40,p $hwndparent,t "${verb}",t $\'${app}$\',t $\'${param}$\',t "${workdir}",i ${show})p.r0'
    System::Call 'shell32::ShellExecuteEx(t)(pr0)i.r1 ?e' ; (t) to trigger A/W selection
    ${If} $1 <> 0
        System::Call '*$0(is,i,p,p,p,p,p,p,p,p,p,p,p,p,p.r1)' ;stack value not really used, just a fancy pop ;)
        System::Call 'kernel32::WaitForSingleObject(pr1,i-1)'
        System::Call 'kernel32::GetExitCodeProcess(pr1,*i.s)'
        System::Call 'kernel32::CloseHandle(pr1)'
    ${EndIf}
    System::Free $0
    !if "${exitoutvar}" == ""
        pop $0
    !endif
    System::Store L
    !if "${exitoutvar}" != ""
        pop ${exitoutvar}
    !endif
!macroend

; --------------------------------------------------------------------------------

Var /GLOBAL OldInstallDir

!macro NSIS_HOOK_PREINSTALL
    SetShellVarContext all
    ${If} ${FileExists} "$SMPROGRAMS\${PRODUCTNAME}.lnk"
        UserInfo::GetAccountType
        Pop $0
        ${If} $0 != "Admin"
            MessageBox MB_ICONINFORMATION|MB_OK "An old installation of the Modrinth App was detected that requires administrator permission to update from. You will be prompted with an admin prompt shortly."
        ${EndIf}

        ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
        ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"

        ReadRegStr $OldInstallDir SHCTX "${UNINSTKEY}" "InstallLocation"
        StrCpy $OldInstallDir $OldInstallDir "" 1
        StrCpy $OldInstallDir $OldInstallDir -1 ""

        DetailPrint "Executing $R1"
        !insertmacro ShellExecWait "runas" '$R1' '/P _?=$4' "" ${SW_SHOW} $3
        ${If} $3 <> 0
            SetErrorLevel $3
            MessageBox MB_ICONEXCLAMATION|MB_OK "Failed to uninstall old global installation"
            Abort
        ${EndIf}
    ${EndIf}
    SetShellVarContext current
!macroend

!macro NSIS_HOOK_POSTINSTALL
    !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$OldInstallDir\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
        !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
        Return
    ${EndIf}
!macroend
