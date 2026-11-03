; Excel - Angerô, edição Windows 7 SP1 64 bits (Inno Setup 6).
; Compilado por montar.ps1. Conteúdo:
;   excel-angero.exe  - compilado com o Rust nightly para x86_64-win7-windows-msvc, runtime C embutido
;                       (não precisa do Visual C++ nem do KB2999226) e SEM atualização automática
;                       (as atualizações publicadas são para Windows 10/11).
;   opengl32.dll + libgallium_wgl.dll - Mesa 26.2.5 (pal1000/mesa-dist-win, MIT): desenha a tela pelo
;                       processador (llvmpipe), para PCs sem placa de vídeo / sem OpenGL 3.3.
; Instala por usuário (não pede senha de administrador).

#ifndef Versao
  #define Versao "0.8.0.0"
#endif
#define Nome "Excel - Angerô"

[Setup]
AppId={{Office-Angero-excel-win7}
AppName={#Nome} (Windows 7)
AppVersion={#Versao}
AppVerName={#Nome} (Windows 7) {#Versao}
AppPublisher=Grupo Angerô
DefaultDirName={localappdata}\Programs\Office Angero\{#Nome} (Windows 7)
DefaultGroupName=Office Angerô
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
OutputDir=saida
OutputBaseFilename=Instalar {#Nome} (Windows 7 64 bits)
SetupIconFile=D:\Projetos\Office-Angero\icones\ico\excel.ico
ChangesAssociations=yes
UninstallDisplayIcon={app}\excel-angero.exe
UninstallDisplayName={#Nome} (Windows 7)
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=6.1sp1
CloseApplications=yes
VersionInfoCompany=Grupo Angerô
VersionInfoProductName={#Nome} (Windows 7)
VersionInfoVersion={#Versao}

[Languages]
Name: "pt"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "associar"; Description: "Usar o {#Nome} como programa padrão para as planilhas"; GroupDescription: "Arquivos:"; Flags: unchecked
Name: "atalho"; Description: "Criar atalho na Área de Trabalho"; GroupDescription: "Atalhos:"

[Files]
Source: "programa\excel-angero.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "programa\opengl32.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "programa\libgallium_wgl.dll"; DestDir: "{app}"; Flags: ignoreversion
; combase.dll não existe no Windows 7: esta só repassa CoTaskMemFree/CoCreateFreeThreadedMarshaler
; para a ole32.dll (no Windows 8+ o sistema usa a dele, que é KnownDLL)
Source: "programa\combase.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "programa\LEIA-ME.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\Projetos\Office-Angero\icones\ico\excel.ico"; DestDir: "{app}"; DestName: "icone.ico"; Flags: ignoreversion

[Icons]
Name: "{group}\{#Nome}"; Filename: "{app}\excel-angero.exe"; IconFilename: "{app}\icone.ico"
Name: "{userdesktop}\{#Nome}"; Filename: "{app}\excel-angero.exe"; IconFilename: "{app}\icone.ico"; Tasks: atalho

[Registry]
Root: HKCU; Subkey: "Software\Grupo Angero\Office Angero\Programas\excel-win7"; ValueType: string; ValueName: "Pasta"; ValueData: "{app}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Grupo Angero\Office Angero\Programas\excel-win7"; ValueType: string; ValueName: "Versao"; ValueData: "{#Versao}"
; associações iguais às do instalador oficial (OpenWithProgids sempre; padrão só com a tarefa)
#define AppPasta_excel "{app}"
#include "D:\Projetos\Office-Angero\instalador\assoc\excel.iss"

[Run]
Filename: "{app}\excel-angero.exe"; Description: "Abrir o {#Nome}"; Flags: nowait postinstall skipifsilent
