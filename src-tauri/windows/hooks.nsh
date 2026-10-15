!macro NSIS_HOOK_PREINSTALL
  ; Terminate any existing Steam Vault instance to prevent file locking
  nsExec::Exec 'taskkill /F /IM "Steam Vault.exe" /T'
  nsExec::Exec 'taskkill /F /IM "steam-vault.exe" /T'
  
  ; If previous version's uninstaller exists in target directory, run it synchronously
  ${If} ${FileExists} "$INSTDIR\uninstall.exe"
    DetailPrint "Uninstalling previous version..."
    ExecWait '"$INSTDIR\uninstall.exe" /S _?=$INSTDIR'
  ${EndIf}
!macroend

!ifdef MULTIUSER_INSTALLMODEPAGE
  !ifdef LANG_KOREAN
    LangString MULTIUSER_TEXT_INSTALLMODE_TITLE ${LANG_KOREAN} "설치 옵션 선택"
    LangString MULTIUSER_TEXT_INSTALLMODE_SUBTITLE ${LANG_KOREAN} "$(^NameDA)을(를) 설치할 사용자를 선택하십시오."
    LangString MULTIUSER_INNERTEXT_INSTALLMODE_TOP ${LANG_KOREAN} "$(^NameDA)을(를) 본인만 사용하도록 설치할지 이 컴퓨터의 모든 사용자를 위해 설치할지 선택하십시오. $(^ClickNext)"
    LangString MULTIUSER_INNERTEXT_INSTALLMODE_ALLUSERS ${LANG_KOREAN} "이 컴퓨터를 사용하는 모든 사용자용으로 설치"
    LangString MULTIUSER_INNERTEXT_INSTALLMODE_CURRENTUSER ${LANG_KOREAN} "나만을 위해 설치"
  !endif
!endif
