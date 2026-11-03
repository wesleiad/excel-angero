"""Confere se um .exe/.dll importa algo que não existe no Windows 7 SP1 (64 bits).

Uso: python -I verificar_win7.py <arquivo.exe> [mais arquivos]
Saída: lista de importações estáticas (as que impedem o programa de abrir) suspeitas.
Importações com atraso (delay-load) são listadas à parte: só falham se forem chamadas.
"""
import sys

import pefile

# DLLs que não existem num Windows 7 SP1 (com ou sem a Atualização de Plataforma)
DLL_AUSENTES = {
    "shcore.dll", "d3d12.dll", "dcomp.dll", "dxcore.dll", "combase.dll", "coremessaging.dll",
    "d3d11on12.dll", "windows.storage.dll", "vcruntime140.dll", "vcruntime140_1.dll",
    "msvcp140.dll", "ucrtbase.dll", "d3dcompiler_47.dll", "dxcompiler.dll", "dxil.dll",
}

# funções que só existem do Windows 8 em diante (ou exigem atualização opcional no 7)
FUNCOES_WIN8 = {
    "GetSystemTimePreciseAsFileTime", "WaitOnAddress", "WakeByAddressSingle", "WakeByAddressAll",
    "CreateFile2", "GetOverlappedResultEx", "SetThreadDescription", "GetThreadDescription",
    "GetCurrentPackageFullName", "GetCurrentPackageId", "GetCurrentPackageFamilyName",
    "PrefetchVirtualMemory", "GetSystemCpuSetInformation", "CopyFile2", "DiscardVirtualMemory",
    "OfferVirtualMemory", "SetProcessMitigationPolicy", "GetProcessMitigationPolicy", "VirtualAlloc2",
    "MapViewOfFile3", "CreateFileMappingFromApp", "GetFirmwareType", "IsWow64Process2",
    "SetThreadInformation", "GetCurrentThreadStackLimits", "SetProcessInformation",
    "InitializeSynchronizationBarrier", "EnterSynchronizationBarrier", "GetFileInformationByName",
    "ProcessPrng", "AddDllDirectory", "SetDefaultDllDirectories", "RemoveDllDirectory",
    "GetDpiForWindow", "GetDpiForSystem", "SetProcessDpiAwarenessContext", "SetThreadDpiAwarenessContext",
    "AdjustWindowRectExForDpi", "GetSystemMetricsForDpi", "EnableNonClientDpiScaling",
    "SystemParametersInfoForDpi", "GetAwarenessFromDpiAwarenessContext", "AreDpiAwarenessContextsEqual",
    "GetWindowDpiAwarenessContext", "GetDpiAwarenessContextForProcess", "GetThreadDpiAwarenessContext",
    "GetPointerInfo", "GetPointerType", "GetPointerFrameInfo", "GetPointerPenInfo", "GetPointerTouchInfo",
    "EnableMouseInPointer", "RegisterPointerInputTarget", "SetDisplayAutoRotationPreferences",
    "IsWindowArranged", "GetDpiForMonitor", "SetProcessDpiAwareness", "GetProcessDpiAwareness",
    "CreateDXGIFactory2", "DXGIGetDebugInterface1", "CoIncrementMTAUsage", "CoDecrementMTAUsage",
    "RoInitialize", "RoUninitialize", "RoGetActivationFactory", "WindowsCreateString",
    "GetPackagesByPackageFamily", "PackageFamilyNameFromFullName", "GetSystemTimeAdjustmentPrecise",
    "QueryInterruptTime", "QueryUnbiasedInterruptTimePrecise", "QueryInterruptTimePrecise",
    "DwmSetIconicThumbnail",
}


def verificar(caminho: str) -> int:
    pe = pefile.PE(caminho)
    oh = pe.OPTIONAL_HEADER
    print(f"{caminho}\n  versão mínima do Windows no cabeçalho: {oh.MajorOperatingSystemVersion}.{oh.MinorOperatingSystemVersion}"
          f" / subsistema {oh.MajorSubsystemVersion}.{oh.MinorSubsystemVersion}")
    problemas = 0
    if (oh.MajorSubsystemVersion, oh.MinorSubsystemVersion) > (6, 1):
        print("  PROBLEMA: subsistema exige Windows mais novo que o 7 (6.1)")
        problemas += 1
    for e in getattr(pe, "DIRECTORY_ENTRY_IMPORT", []):
        dll = e.dll.decode().lower()
        nomes = [i.name.decode() for i in e.imports if i.name]
        ruins = [n for n in nomes if n in FUNCOES_WIN8]
        if dll in DLL_AUSENTES or dll.startswith("api-ms-win-"):
            print(f"  PROBLEMA: DLL {dll} (estática, {len(nomes)} funções)")
            problemas += 1
        for n in ruins:
            print(f"  PROBLEMA: {dll}!{n} (estática)")
            problemas += 1
        print(f"    {dll}: {len(nomes)}")
    for e in getattr(pe, "DIRECTORY_ENTRY_DELAY_IMPORT", []):
        print(f"    (atraso) {e.dll.decode().lower()}: {len(e.imports)}")
    print(f"  {problemas} problema(s)")
    return problemas


if __name__ == "__main__":
    total = sum(verificar(a) for a in sys.argv[1:])
    sys.exit(1 if total else 0)
