# Monta o "Excel - Angerô (Windows 7 64 bits)" — ramo windows-7 do excel-angero.
# Precisa: rustup com nightly + rust-src, Build Tools do Visual Studio (link.exe), Inno Setup 6, Python com pefile.
# Pasta de trabalho: D:\Projetos\Excel-Win7 (target, download do Mesa, programa\, saida\).
#   .\win7\montar.ps1 -Versao 0.8.0.15
param([string]$Versao = "0.8.0.0", [string]$Trabalho = "D:\Projetos\Excel-Win7")
$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent
$alvo = "x86_64-win7-windows-msvc"
New-Item -ItemType Directory -Force "$Trabalho\programa", "$Trabalho\download", "$Trabalho\combase" | Out-Null

# 1) exe: alvo Windows 7 (std recompilada) com o runtime C embutido (sem Visual C++ no PC)
$env:CARGO_TARGET_X86_64_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static"
$env:CARGO_TARGET_DIR = "$Trabalho\target"
Push-Location $repo
cargo +nightly build --release -Z build-std=std,panic_abort --target $alvo -p gridcraft --bin excel-angero --locked
if ($LASTEXITCODE) { throw "cargo falhou" }
Pop-Location
Copy-Item "$Trabalho\target\$alvo\release\excel-angero.exe" "$Trabalho\programa\" -Force

# 2) Mesa 26.2.5 MSVC (o pacote 26.2.6 MSVC saiu sem opengl32.dll). tar do Windows abre o .7z (BCJ2).
$mesa = "$Trabalho\download\mesa3d-26.2.5-release-msvc.7z"
if (-not (Test-Path $mesa)) {
    curl.exe -sL --ssl-no-revoke -o $mesa "https://github.com/pal1000/mesa-dist-win/releases/download/26.2.5/mesa3d-26.2.5-release-msvc.7z"
}
Push-Location "$Trabalho\download"
New-Item -ItemType Directory -Force mesa | Out-Null
tar.exe -xf $mesa -C mesa x64/opengl32.dll x64/libgallium_wgl.dll
Pop-Location
Copy-Item "$Trabalho\download\mesa\x64\*.dll" "$Trabalho\programa\" -Force

# 3) combase.dll de repasse para a ole32.dll (o windows-core importa 2 funções da combase.dll)
$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -property installationPath
$bin = (Get-ChildItem "$vs\VC\Tools\MSVC\*\bin\Hostx64\x64" | Select-Object -Last 1).FullName
Copy-Item "$PSScriptRoot\combase.def" "$Trabalho\combase\" -Force
Push-Location "$Trabalho\combase"
& "$bin\lib.exe" -nologo -def:combase.def -machine:x64 -out:combase_imp.lib
& "$bin\link.exe" -nologo -dll -noentry -nodefaultlib -machine:x64 -subsystem:windows,6.01 combase_imp.exp -out:combase.dll
Pop-Location
Copy-Item "$Trabalho\combase\combase.dll" "$Trabalho\programa\" -Force
Copy-Item "$PSScriptRoot\LEIA-ME.txt" "$Trabalho\programa\" -Force

# 4) confere importações que não existem no Windows 7
python -I "$PSScriptRoot\verificar_win7.py" "$Trabalho\programa\excel-angero.exe"
if ($LASTEXITCODE) { throw "o exe importa algo que não existe no Windows 7" }

# 5) instalador
Copy-Item "$PSScriptRoot\instalar-excel-win7.iss" "$Trabalho\" -Force
Push-Location $Trabalho
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" /Q "/DVersao=$Versao" instalar-excel-win7.iss
if ($LASTEXITCODE) { throw "Inno Setup falhou" }
Pop-Location
Get-ChildItem "$Trabalho\saida"
