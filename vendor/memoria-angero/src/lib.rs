//! Office Angerô: chamadas ao Windows que precisam de `unsafe`, isoladas aqui porque o workspace
//! proíbe `unsafe` (`forbid(unsafe_code)`).

/// Área livre do monitor principal (sem a barra de tarefas), em pontos lógicos: (x, y, largura, altura).
/// Office Angerô: usada para a janela abrir cabendo na tela em notebooks e telas com zoom de 125-150 %.
#[cfg(target_os = "windows")]
pub fn area_trabalho() -> Option<(f32, f32, f32, f32)> {
    #[repr(C)]
    struct Retangulo {
        esquerda: i32,
        topo: i32,
        direita: i32,
        baixo: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(acao: u32, param: u32, dados: *mut Retangulo, ini: u32) -> i32;
        fn GetDC(janela: isize) -> isize;
        fn ReleaseDC(janela: isize, dc: isize) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(nome: *const u16) -> isize;
        fn GetProcAddress(modulo: isize, nome: *const u8) -> Option<unsafe extern "system" fn() -> u32>;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn GetDeviceCaps(dc: isize, indice: i32) -> i32;
    }
    const SPI_GETWORKAREA: u32 = 0x0030;
    const LOGPIXELSX: i32 = 88;
    // GetDpiForSystem só existe a partir do Windows 10 1607: procurada em tempo de execução (no
    // Windows 7 a importação direta impediria o programa de abrir); sem ela, o DPI da tela via GDI.
    let dpi_do_sistema = || -> u32 {
        let user32: Vec<u16> = "user32.dll\0".encode_utf16().collect();
        // SAFETY: nomes terminados em zero; a função achada não recebe argumentos e devolve u32.
        unsafe {
            let m = GetModuleHandleW(user32.as_ptr());
            if m != 0 {
                if let Some(f) = GetProcAddress(m, c"GetDpiForSystem".as_ptr().cast()) {
                    return f();
                }
            }
            let dc = GetDC(0);
            if dc == 0 {
                return 96;
            }
            let dpi = GetDeviceCaps(dc, LOGPIXELSX);
            ReleaseDC(0, dc);
            dpi.max(0) as u32
        }
    };
    let mut r = Retangulo { esquerda: 0, topo: 0, direita: 0, baixo: 0 };
    // SAFETY: `r` é um RECT válido; as funções só leem configurações do sistema.
    let (ok, dpi) = unsafe { (SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut r, 0), dpi_do_sistema()) };
    if ok == 0 || dpi == 0 || r.direita <= r.esquerda || r.baixo <= r.topo {
        return None;
    }
    // processo com ou sem reconhecimento de DPI: pixels e DPI vêm na mesma escala, a divisão dá pontos lógicos
    let k = dpi as f32 / 96.0;
    Some((
        r.esquerda as f32 / k,
        r.topo as f32 / k,
        (r.direita - r.esquerda) as f32 / k,
        (r.baixo - r.topo) as f32 / k,
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn area_trabalho() -> Option<(f32, f32, f32, f32)> {
    None
}
