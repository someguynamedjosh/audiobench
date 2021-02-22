Unicode True

Name "Audiobench"
OutFile "AudiobenchInstaller.exe"
InstallDir "$PROGRAMFILES64\Audiobench"
RequestExecutionLevel admin
CRCCheck On

!include MUI2.nsh

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE.md"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Function .onInit
UserInfo::GetAccountType
pop $0
${If} $0 != "admin" ;Require admin rights on NT4+
    MessageBox mb_iconstop "Administrator rights required!"
    SetErrorLevel 740 ;ERROR_ELEVATION_REQUIRED
    Quit
${EndIf}
FunctionEnd

!include LogicLib.nsh
!include WinCore.nsh
!ifndef NSIS_CHAR_SIZE
!define NSIS_CHAR_SIZE 1
!endif

Function RegAppendString
System::Store S
Pop $R0 ; append
Pop $R1 ; separator
Pop $R2 ; reg value
Pop $R3 ; reg path
Pop $R4 ; reg hkey
System::Call 'ADVAPI32::RegCreateKey(i$R4,tR3,*i.r1)i.r0'
${If} $0 = 0
    System::Call 'ADVAPI32::RegQueryValueEx(ir1,tR2,i0,*i.r2,i0,*i0r3)i.r0'
    ${If} $0 <> 0
        StrCpy $2 ${REG_SZ}
        StrCpy $3 0
    ${EndIf}
    StrLen $4 $R0
    StrLen $5 $R1
    IntOp $4 $4 + $5
    IntOp $4 $4 + 1 ; For \0
    !if ${NSIS_CHAR_SIZE} > 1
        IntOp $4 $4 * ${NSIS_CHAR_SIZE}
    !endif
    IntOp $4 $4 + $3
    System::Alloc $4
    System::Call 'ADVAPI32::RegQueryValueEx(ir1,tR2,i0,i0,isr9,*ir4r4)i.r0'
    ${If} $0 = 0
    ${OrIf} $0 = ${ERROR_FILE_NOT_FOUND}
        System::Call 'KERNEL32::lstrlen(t)(ir9)i.r0'
        ${If} $0 <> 0
            System::Call 'KERNEL32::lstrcat(t)(ir9,tR1)'
        ${EndIf}
        System::Call 'KERNEL32::lstrcat(t)(ir9,tR0)'
        System::Call 'KERNEL32::lstrlen(t)(ir9)i.r0'
        IntOp $0 $0 + 1
        !if ${NSIS_CHAR_SIZE} > 1
            IntOp $0 $0 * ${NSIS_CHAR_SIZE}
        !endif
        System::Call 'ADVAPI32::RegSetValueEx(ir1,tR2,i0,ir2,ir9,ir0)i.r0'
    ${EndIf}
    System::Free $9
    System::Call 'ADVAPI32::RegCloseKey(ir1)'
${EndIf}
Push $0
System::Store L
FunctionEnd

Section
    Push ${HKEY_LOCAL_MACHINE}
    Push "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    Push "Path"
    Push ";"
    Push "$INSTDIR\julia\bin\"
    Call RegAppendString
    Pop $0
    DetailPrint RegAppendString:Error=$0

    SetOutPath "$INSTDIR"
    File /oname=Audiobench.exe "..\..\artifacts\bin\Audiobench_Windows_x64_Standalone.exe"
    File /oname=audiobench_clib.dll "..\..\artifacts\bin\audiobench_clib.dll"
    SetOutPath "$PROGRAMFILES64\Common Files\VST3\"
    File /oname=Audiobench.vst3 /r "..\..\artifacts\bin\Audiobench_Windows_x64_VST3.vst3"
    SetOutPath "$INSTDIR\julia"
    File /r "..\..\dependencies\julia\"
    WriteUninstaller "$INSTDIR\uninstaller.exe"

    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Audiobench" \
                 "DisplayName" "Audiobench Modular Synthesizer"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Audiobench" \
                 "UninstallString" "$\"$INSTDIR\uninstall.exe$\""

    CreateShortcut "$SMPROGRAMS\Audiobench (Standalone).lnk" "$INSTDIR\Audiobench.exe"

    MessageBox MB_YESNO|MB_ICONQUESTION "Installation complete! You must reboot your computer before you can use Audiobench, would you like to reboot right now?" IDNO +2
    Reboot
SectionEnd

Section "Uninstall"
    Delete $INSTDIR\uninstaller.exe
    Delete $INSTDIR\Audiobench.exe
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Audiobench"
    RMDir $INSTDIR
SectionEnd
